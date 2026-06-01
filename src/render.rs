use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Write};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::model::*;
use crate::package_path;
use crate::validation::ValidationError;

use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Feature, Features, Library, LibraryExt, World};
use typst_kit::fonts::FontSearcher;
use typst_pdf::PdfOptions;
use typst_syntax::ast;
use typst_syntax::{SyntaxKind, SyntaxNode};

/// Which physical document(s) a render produces.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edition {
    /// One HTML document per source document (the source-document invariant).
    Html,
    /// One combined PDF document assembling the selected region.
    Pdf,
}

/// The publication region a render covers. The edition decides whether that
/// region is emitted as per-source pages (HTML) or one assembled document (PDF).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderTarget {
    WholePublication,
    NamedScope(ScopeId),
    ScopeUnion(Vec<ScopeId>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    pub source_root: PathBuf,
    pub output_dir: PathBuf,
    pub artifact_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderedArtifacts {
    pub html_paths: Vec<PathBuf>,
    pub pdf_path: Option<PathBuf>,
    pub entrypoint_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum RenderError {
    ValidationFailed { errors: Vec<ValidationError> },
    UnknownScope { scope_id: ScopeId },
    CreateOutputDir { path: PathBuf, source: io::Error },
    AssemblyWrite { path: PathBuf, source: io::Error },
    PagedCompilation { diagnostics: Vec<String> },
    PdfExport { diagnostics: Vec<String> },
    PdfWrite { path: PathBuf, source: io::Error },
    HtmlCompilation { diagnostics: Vec<String> },
    HtmlExport { diagnostics: Vec<String> },
    CssRead { path: PathBuf, source: io::Error },
    InvalidCssPath { path: String },
    InvalidCssPayload { value: Value },
    MissingHtmlHead,
    HtmlWrite { path: PathBuf, source: io::Error },
}

impl Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::ValidationFailed { errors } => {
                write!(f, "publication failed validation before render: {errors:?}")
            }
            RenderError::UnknownScope { scope_id } => {
                write!(f, "unknown scope id: {scope_id}")
            }
            RenderError::CreateOutputDir { path, source } => write!(
                f,
                "failed to create render output directory {}: {source}",
                path.display()
            ),
            RenderError::AssemblyWrite { path, source } => {
                write!(
                    f,
                    "failed to write Typst source {}: {source}",
                    path.display()
                )
            }
            RenderError::PagedCompilation { diagnostics } => write!(
                f,
                "failed to compile combined Typst document: {}",
                diagnostics.join("; ")
            ),
            RenderError::PdfExport { diagnostics } => {
                write!(f, "failed to export PDF: {}", diagnostics.join("; "))
            }
            RenderError::PdfWrite { path, source } => {
                write!(f, "failed to write PDF {}: {source}", path.display())
            }
            RenderError::HtmlCompilation { diagnostics } => write!(
                f,
                "failed to compile HTML Typst document: {}",
                diagnostics.join("; ")
            ),
            RenderError::HtmlExport { diagnostics } => {
                write!(f, "failed to export HTML: {}", diagnostics.join("; "))
            }
            RenderError::CssRead { path, source } => {
                write!(f, "failed to read CSS file {}: {source}", path.display())
            }
            RenderError::InvalidCssPath { path } => {
                write!(
                    f,
                    "CSS path must be relative and stay within the source root: {path}"
                )
            }
            RenderError::InvalidCssPayload { value } => {
                write!(f, "CSS payload must be a file path string, got {value:?}")
            }
            RenderError::MissingHtmlHead => {
                write!(f, "HTML export did not contain a </head> insertion point")
            }
            RenderError::HtmlWrite { path, source } => {
                write!(f, "failed to write HTML {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for RenderError {}

const ENTRYPOINT_VPATH: &str = "__publisher_entrypoint.typ";

/// Render `target` in `edition`, writing artifacts under `options.output_dir`.
pub fn render_target(
    publication: &Publication,
    target: &RenderTarget,
    edition: Edition,
    options: &RenderOptions,
) -> Result<RenderedArtifacts, RenderError> {
    let report = publication.validate();
    if !report.is_ok() {
        return Err(RenderError::ValidationFailed {
            errors: report.errors,
        });
    }
    ensure_target_scopes_exist(publication, target)?;

    fs::create_dir_all(&options.output_dir).map_err(|source| RenderError::CreateOutputDir {
        path: options.output_dir.clone(),
        source,
    })?;

    let nodes = select_nodes(publication, target);

    match edition {
        Edition::Html => render_html_pages(publication, &nodes, options),
        Edition::Pdf => render_pdf(publication, &nodes, options),
    }
}

fn ensure_target_scopes_exist(
    publication: &Publication,
    target: &RenderTarget,
) -> Result<(), RenderError> {
    let ids: Vec<&ScopeId> = match target {
        RenderTarget::WholePublication => Vec::new(),
        RenderTarget::NamedScope(id) => vec![id],
        RenderTarget::ScopeUnion(ids) => ids.iter().collect(),
    };
    for id in ids {
        if publication.scope(id).is_none() {
            return Err(RenderError::UnknownScope {
                scope_id: id.clone(),
            });
        }
    }
    Ok(())
}

/// Nodes covered by `target`, preserving publication-graph order.
fn select_nodes<'a>(publication: &'a Publication, target: &RenderTarget) -> Vec<&'a Node> {
    publication
        .nodes
        .iter()
        .filter(|node| target_covers(publication, target, &node.id))
        .collect()
}

fn target_covers(publication: &Publication, target: &RenderTarget, node_id: &NodeId) -> bool {
    match target {
        RenderTarget::WholePublication => true,
        RenderTarget::NamedScope(id) => publication
            .scope(id)
            .is_some_and(|scope| scope.covers(publication, node_id)),
        RenderTarget::ScopeUnion(ids) => ids.iter().any(|id| {
            publication
                .scope(id)
                .is_some_and(|scope| scope.covers(publication, node_id))
        }),
    }
}

// ---------------------------------------------------------------------------
// HTML edition: one document per source.
// ---------------------------------------------------------------------------

fn render_html_pages(
    publication: &Publication,
    nodes: &[&Node],
    options: &RenderOptions,
) -> Result<RenderedArtifacts, RenderError> {
    let mut html_paths = Vec::new();
    let labels = publication_label_targets(publication);

    for node in nodes {
        let page_source = standalone_html_source(publication, node, &labels);
        let world = RenderWorld::new(
            &options.source_root,
            &node.source_path,
            [(node.source_path.clone(), page_source)]
                .into_iter()
                .collect(),
            Edition::Html,
        );

        let document = typst::compile::<typst_html::HtmlDocument>(&world)
            .output
            .map_err(|diagnostics| RenderError::HtmlCompilation {
                diagnostics: format_diagnostics(diagnostics.iter()),
            })?;
        let html = typst_html::html(&document).map_err(|diagnostics| RenderError::HtmlExport {
            diagnostics: format_diagnostics(diagnostics.iter()),
        })?;
        let html = inject_css(publication, node, options, html)?;

        let html_path = options.output_dir.join(&node.html_route);
        if let Some(parent) = html_path.parent() {
            fs::create_dir_all(parent).map_err(|source| RenderError::CreateOutputDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&html_path, html).map_err(|source| RenderError::HtmlWrite {
            path: html_path.clone(),
            source,
        })?;
        html_paths.push(html_path);
    }

    Ok(RenderedArtifacts {
        html_paths,
        ..Default::default()
    })
}

fn inject_css(
    publication: &Publication,
    node: &Node,
    options: &RenderOptions,
    html: String,
) -> Result<String, RenderError> {
    let payloads = publication.payloads_for(&node.id, "css").map_err(|error| {
        RenderError::ValidationFailed {
            errors: vec![error],
        }
    })?;
    if payloads.is_empty() {
        return Ok(html);
    }

    let mut styles = String::new();
    for payload in payloads {
        let Value::String(css_path) = &payload.value else {
            return Err(RenderError::InvalidCssPayload {
                value: payload.value.clone(),
            });
        };
        let path = resolve_declared_file(&options.source_root, &payload.source_path, css_path)?;
        let css = fs::read_to_string(&path).map_err(|source| RenderError::CssRead {
            path: path.clone(),
            source,
        })?;
        writeln!(
            styles,
            "<style data-publisher-css=\"{}\">\n{}\n</style>",
            html_attr_text(css_path),
            css
        )
        .unwrap();
    }

    let Some(head_end) = html.find("</head>") else {
        return Err(RenderError::MissingHtmlHead);
    };
    let mut out = String::with_capacity(html.len() + styles.len());
    out.push_str(&html[..head_end]);
    out.push_str(&styles);
    out.push_str(&html[head_end..]);
    Ok(out)
}

fn resolve_declared_file(
    source_root: &Path,
    declaring_source: &str,
    declared_path: &str,
) -> Result<PathBuf, RenderError> {
    let declared = Path::new(declared_path);
    if declared.is_absolute() {
        return Err(RenderError::InvalidCssPath {
            path: declared_path.to_string(),
        });
    }

    let mut relative = Path::new(declaring_source)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();
    for component in declared.components() {
        match component {
            Component::Normal(segment) => relative.push(segment),
            Component::CurDir => {}
            Component::ParentDir => {
                if !relative.pop() {
                    return Err(RenderError::InvalidCssPath {
                        path: declared_path.to_string(),
                    });
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(RenderError::InvalidCssPath {
                    path: declared_path.to_string(),
                });
            }
        }
    }

    Ok(source_root.join(relative))
}

/// A source compiled as its own HTML page. Outline and bibliography stay as
/// authored (each page has at most one bibliography and its own headings);
/// cross-source references are lowered to links and cross-source numbering is
/// seeded when the active scope spans several sources.
fn standalone_html_source(
    publication: &Publication,
    node: &Node,
    labels: &BTreeMap<String, LabelTarget>,
) -> String {
    let mut source = node.authored_source.typst.clone();
    source = seed_heading_counter(publication, node, &source);
    replace_references(&source, |label| {
        let target = labels.get(label)?;
        if target.node_id == node.id {
            return None; // same document: native Typst lookup resolves it
        }
        let route = relative_route(&node.html_route, &target.route);
        let display = reference_display(publication, node, target, label);
        Some(format!(
            "#link(\"{}#{}\")[{}]",
            typst_string_text(&route),
            typst_string_text(label),
            escape_content_text(&display)
        ))
    })
}

// ---------------------------------------------------------------------------
// PDF edition: assemble the region into one compiled document.
// ---------------------------------------------------------------------------

fn render_pdf(
    publication: &Publication,
    nodes: &[&Node],
    options: &RenderOptions,
) -> Result<RenderedArtifacts, RenderError> {
    let mut overlays: BTreeMap<String, String> = BTreeMap::new();
    let labels = publication_label_targets(publication);

    // Transform each included source: bound outlines to the active scope, strip
    // authored bibliographies (one consolidated call is emitted below), and keep
    // references that resolve within the assembly native.
    for node in nodes {
        let scope_label = nearest_publication_scope(publication, node)
            .map(|scope| entrypoint_file_stem(scope.id.as_str()));
        let mut source = node.authored_source.typst.clone();
        source = strip_bibliography_calls(&source);
        if let Some(scope_label) = &scope_label {
            source = bound_outlines(&source, scope_label);
        }
        source = replace_references(&source, |label| {
            let target = labels.get(label)?;
            if target.node_id == node.id {
                return None; // same source: native Typst lookup resolves it
            }
            // A cross-source reference. Even when the target is elsewhere in the
            // assembly, a native `@label` to an unnumbered heading fails to
            // compile, so lower to display text (carrying scope context).
            Some(escape_content_text(&reference_display(
                publication,
                node,
                target,
                label,
            )))
        });
        overlays.insert(node.source_path.clone(), source);
    }

    let entrypoint = build_pdf_entrypoint(publication, nodes);
    overlays.insert(ENTRYPOINT_VPATH.to_string(), entrypoint.clone());

    let typst_path = options
        .output_dir
        .join(format!("{}.typ", options.artifact_name));
    fs::write(&typst_path, &entrypoint).map_err(|source| RenderError::AssemblyWrite {
        path: typst_path.clone(),
        source,
    })?;

    let world = RenderWorld::new(
        &options.source_root,
        ENTRYPOINT_VPATH,
        overlays,
        Edition::Pdf,
    );
    let document = typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|diagnostics| RenderError::PagedCompilation {
            diagnostics: format_diagnostics(diagnostics.iter()),
        })?;
    let pdf = typst_pdf::pdf(&document, &PdfOptions::default()).map_err(|diagnostics| {
        RenderError::PdfExport {
            diagnostics: format_diagnostics(diagnostics.iter()),
        }
    })?;

    let pdf_path = options
        .output_dir
        .join(format!("{}.pdf", options.artifact_name));
    fs::write(&pdf_path, pdf).map_err(|source| RenderError::PdfWrite {
        path: pdf_path.clone(),
        source,
    })?;

    Ok(RenderedArtifacts {
        html_paths: Vec::new(),
        pdf_path: Some(pdf_path),
        entrypoint_path: Some(typst_path),
    })
}

/// Build the combined entrypoint: scope-boundary markers around each contiguous
/// run of sources sharing a scope, `#include`s in graph order, and exactly one
/// consolidated bibliography over the union of cited sources.
fn build_pdf_entrypoint(publication: &Publication, nodes: &[&Node]) -> String {
    let mut out = String::new();
    let mut open_scope: Option<String> = None;

    for (index, node) in nodes.iter().enumerate() {
        let scope_label = nearest_publication_scope(publication, node)
            .map(|scope| entrypoint_file_stem(scope.id.as_str()));

        if open_scope != scope_label {
            if let Some(label) = &open_scope {
                writeln!(out, "#metadata(none) <scope-end-{label}>").unwrap();
            }
            if let Some(label) = &scope_label {
                writeln!(out, "#metadata(none) <scope-start-{label}>").unwrap();
            }
            open_scope = scope_label;
        }

        if index > 0 {
            writeln!(out, "#pagebreak()").unwrap();
        }
        writeln!(out, "#include \"{}\"", node.source_path).unwrap();
    }

    if let Some(label) = &open_scope {
        writeln!(out, "#metadata(none) <scope-end-{label}>").unwrap();
    }

    let bibliography_sources = consolidated_bibliography_sources(publication, nodes);
    if let Some(call) = consolidated_bibliography_call(&bibliography_sources) {
        writeln!(out).unwrap();
        writeln!(out, "{call}").unwrap();
    }

    out
}

/// Resolve every authored bibliography source in `nodes` to an entrypoint-root
/// relative path and de-duplicate, preserving order.
fn consolidated_bibliography_sources(publication: &Publication, nodes: &[&Node]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut sources = Vec::new();
    for node in nodes {
        for raw in bibliography_sources_for_node(publication, node) {
            let resolved = resolve_relative(&node.source_path, &raw);
            if seen.insert(resolved.clone()) {
                sources.push(resolved);
            }
        }
    }
    sources
}

fn consolidated_bibliography_call(sources: &[String]) -> Option<String> {
    match sources {
        [] => None,
        [single] => Some(format!("#bibliography(\"{}\")", typst_string_text(single))),
        many => {
            let list = many
                .iter()
                .map(|source| format!("\"{}\"", typst_string_text(source)))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("#bibliography(({list}))"))
        }
    }
}

/// Resolve `relative` against the directory of `source_path`, collapsing `..`,
/// and return the entrypoint-root relative result.
fn resolve_relative(source_path: &str, relative: &str) -> String {
    let mut components: Vec<&str> = Path::new(source_path)
        .parent()
        .map(|parent| parent.to_str().unwrap_or("").split('/'))
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty())
        .collect();

    for segment in relative.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            other => components.push(other),
        }
    }

    components.join("/")
}

// ---------------------------------------------------------------------------
// Source transformations.
// ---------------------------------------------------------------------------

/// Remove every authored `#bibliography(...)` call; the assembly emits one
/// consolidated bibliography instead (Typst rejects multiple per document).
fn strip_bibliography_calls(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0;

    while index < source.len() {
        if let Some(open_paren) = bibliography_call_at(source, index)
            && let Some(end) = find_call_end(source, open_paren)
        {
            index = end;
            continue;
        }

        let ch = source[index..]
            .chars()
            .next()
            .expect("index is within source");
        out.push(ch);
        index += ch.len_utf8();
    }

    out
}

/// Bound every authored `#outline(...)` to the active scope's start/end markers
/// so it lists only that scope's headings inside a larger assembled document.
fn bound_outlines(source: &str, scope_label: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0;

    while index < source.len() {
        if let Some(open_paren) = outline_call_at(source, index)
            && let Some(end) = find_call_end(source, open_paren)
        {
            let inner = &source[open_paren + 1..end - 1];
            out.push_str("#outline(");
            out.push_str(&bound_outline_args(inner, scope_label));
            out.push(')');
            index = end;
            continue;
        }

        let ch = source[index..]
            .chars()
            .next()
            .expect("index is within source");
        out.push(ch);
        index += ch.len_utf8();
    }

    out
}

fn bound_outline_args(inner: &str, scope_label: &str) -> String {
    let bound = format!(
        ".after(<scope-start-{scope_label}>, inclusive: false).before(<scope-end-{scope_label}>, inclusive: false)"
    );

    if let Some(pos) = inner.find("target:") {
        let after = &inner[pos + "target:".len()..];
        let expr_end = top_level_comma(after).unwrap_or(after.len());
        let expr = after[..expr_end].trim();
        let rest = &after[expr_end..];
        let prefix = &inner[..pos];
        format!("{prefix}target: ({expr}){bound}{rest}")
    } else {
        let sep = if inner.trim().is_empty() { "" } else { ", " };
        format!("target: heading.where(level: 1){bound}{sep}{inner}")
    }
}

/// Byte offset of the first comma at paren/bracket depth zero, if any.
fn top_level_comma(source: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in source.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => return Some(offset),
            _ => {}
        }
    }
    None
}

/// Scan `source` for `@label` references, replacing each with the result of
/// `replace(label)`; `None` keeps the original `@label`.
fn replace_references(source: &str, mut replace: impl FnMut(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < source.len() {
        let ch = source[index..]
            .chars()
            .next()
            .expect("index is within source");

        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += ch.len_utf8();
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            index += ch.len_utf8();
            continue;
        }

        if ch == '@' {
            let label_start = index + ch.len_utf8();
            if let Some((label, end)) = take_label(&source[label_start..]) {
                match replace(label) {
                    Some(replacement) => out.push_str(&replacement),
                    None => {
                        out.push('@');
                        out.push_str(label);
                    }
                }
                index = label_start + end;
                continue;
            }
        }

        out.push(ch);
        index += ch.len_utf8();
    }

    out
}

fn seed_heading_counter(publication: &Publication, node: &Node, source: &str) -> String {
    let Some(seed) = heading_counter_seed(publication, node) else {
        return source.to_string();
    };
    format!("#counter(heading).update({seed})\n\n{source}")
}

fn heading_counter_seed(publication: &Publication, node: &Node) -> Option<usize> {
    if !source_sets_heading_numbering(&node.authored_source.typst) {
        return None;
    }

    let spine = publication.spine_for(&node.id).ok()?;
    let scope = spine
        .scopes
        .iter()
        .rev()
        .filter_map(|scope_id| publication.scope(scope_id))
        .find(|scope| {
            scope.kind == ScopeKind::Publication
                && numbered_scope_node_count(publication, scope) > 1
        })?;

    let seed = publication
        .nodes
        .iter()
        .take_while(|candidate| candidate.id != node.id)
        .filter(|candidate| scope.covers(publication, &candidate.id))
        .filter(|candidate| source_sets_heading_numbering(&candidate.authored_source.typst))
        .map(|candidate| top_level_heading_count(&candidate.authored_source.typst))
        .sum();

    (seed > 0).then_some(seed)
}

fn numbered_scope_node_count(publication: &Publication, scope: &Scope) -> usize {
    publication
        .nodes
        .iter()
        .filter(|node| scope.covers(publication, &node.id))
        .filter(|node| source_sets_heading_numbering(&node.authored_source.typst))
        .count()
}

fn source_sets_heading_numbering(source: &str) -> bool {
    source.contains("#set heading(") && source.contains("numbering")
}

fn top_level_heading_count(source: &str) -> usize {
    let syntax = typst_syntax::parse(source);
    let mut count = 0;
    count_top_level_headings(&syntax, &mut count);
    count
}

fn count_top_level_headings(node: &SyntaxNode, count: &mut usize) {
    if node.kind() == SyntaxKind::Heading
        && let Some(heading) = node.cast::<ast::Heading>()
        && heading.depth().get() == 1
    {
        *count += 1;
    }
    for child in node.children() {
        count_top_level_headings(child, count);
    }
}

// ---------------------------------------------------------------------------
// Scope and label helpers.
// ---------------------------------------------------------------------------

/// Explicit publication scopes, in declaration order. Used by the CLI/inspect
/// surface to enumerate render targets.
pub fn publication_scope_ids(publication: &Publication) -> Vec<ScopeId> {
    publication
        .scopes
        .iter()
        .filter(|scope| !scope.implicit && scope.kind == ScopeKind::Publication)
        .map(|scope| scope.id.clone())
        .collect()
}

fn entrypoint_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn nearest_publication_scope<'a>(publication: &'a Publication, node: &Node) -> Option<&'a Scope> {
    let spine = publication.spine_for(&node.id).ok()?;
    spine
        .scopes
        .iter()
        .rev()
        .filter_map(|scope_id| publication.scope(scope_id))
        .find(|scope| scope.kind == ScopeKind::Publication)
}

fn bibliography_sources_for_node(publication: &Publication, node: &Node) -> Vec<String> {
    publication
        .properties_for_node(&node.id)
        .into_iter()
        .filter(|property| property.key == "bibliography-source")
        .filter_map(|property| match &property.value {
            Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LabelTarget {
    node_id: NodeId,
    route: String,
    title: Option<String>,
}

fn publication_label_targets(publication: &Publication) -> BTreeMap<String, LabelTarget> {
    let mut labels = BTreeMap::new();

    for node in &publication.nodes {
        let mut node_labels = Vec::new();
        let syntax = typst_syntax::parse(&node.authored_source.typst);
        collect_labels(&syntax, &mut node_labels);

        for label in node_labels {
            labels.entry(label).or_insert_with(|| LabelTarget {
                node_id: node.id.clone(),
                route: node.html_route.clone(),
                title: title_for_node(publication, &node.id),
            });
        }
    }

    labels
}

fn collect_labels(node: &SyntaxNode, labels: &mut Vec<String>) {
    if node.kind() == SyntaxKind::Label
        && let Some(label) = node.cast::<ast::Label>()
    {
        labels.push(label.get().to_string());
    }
    for child in node.children() {
        collect_labels(child, labels);
    }
}

fn reference_display(
    publication: &Publication,
    origin: &Node,
    target: &LabelTarget,
    label: &str,
) -> String {
    let title = target.title.clone().unwrap_or_else(|| label.to_string());
    if let Some(scope_title) = cross_scope_reference_title(publication, &origin.id, &target.node_id)
        && scope_title != title
    {
        return format!("{title}, {scope_title}");
    }
    title
}

fn cross_scope_reference_title(
    publication: &Publication,
    origin: &NodeId,
    target: &NodeId,
) -> Option<String> {
    let origin_spine = publication.spine_for(origin).ok()?;
    let target_spine = publication.spine_for(target).ok()?;
    let origin_scopes: BTreeSet<_> = origin_spine.scopes.into_iter().collect();

    target_spine
        .scopes
        .iter()
        .filter(|scope_id| !origin_scopes.contains(*scope_id))
        .filter_map(|scope_id| publication.scope(scope_id))
        .find(|scope| scope.kind == ScopeKind::Publication && scope.name.is_some())
        .and_then(|scope| scope.name.clone())
}

fn relative_route(from_route: &str, to_route: &str) -> String {
    let from_parent = Path::new(from_route)
        .parent()
        .map(route_components)
        .unwrap_or_default();
    let to_components = route_components(Path::new(to_route));

    let mut common = 0usize;
    while common < from_parent.len()
        && common < to_components.len()
        && from_parent[common] == to_components[common]
    {
        common += 1;
    }

    let mut relative = Vec::new();
    for _ in common..from_parent.len() {
        relative.push("..".to_string());
    }
    relative.extend(to_components[common..].iter().cloned());

    if relative.is_empty() {
        Path::new(to_route)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(to_route)
            .to_string()
    } else {
        relative.join("/")
    }
}

fn route_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => {
                value.to_str().map(std::string::ToString::to_string)
            }
            _ => None,
        })
        .collect()
}

fn title_for_node(publication: &Publication, node_id: &NodeId) -> Option<String> {
    publication
        .properties_for_node(node_id)
        .iter()
        .find(|property| property.key == "title")
        .and_then(|property| match &property.value {
            Value::String(value) => Some(value.clone()),
            _ => None,
        })
}

// ---------------------------------------------------------------------------
// Lexical helpers shared by the transformations.
// ---------------------------------------------------------------------------

fn bibliography_call_at(source: &str, index: usize) -> Option<usize> {
    call_open_paren(source, index, "#bibliography")
}

fn outline_call_at(source: &str, index: usize) -> Option<usize> {
    call_open_paren(source, index, "#outline")
}

fn call_open_paren(source: &str, index: usize, name: &str) -> Option<usize> {
    if !source[index..].starts_with(name) {
        return None;
    }
    let mut open_paren = index + name.len();
    while let Some(ch) = source[open_paren..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        open_paren += ch.len_utf8();
    }
    source[open_paren..].starts_with('(').then_some(open_paren)
}

fn find_call_end(source: &str, open_paren: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in source[open_paren..].char_indices() {
        let index = open_paren + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index + ch.len_utf8());
                }
            }
            _ => {}
        }
    }

    None
}

fn take_label(source: &str) -> Option<(&str, usize)> {
    let mut end = 0;
    for ch in source.chars() {
        if !is_label_char(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    (end > 0).then(|| (&source[..end], end))
}

fn is_label_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '/')
}

fn typst_string_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn html_attr_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape text for use inside a Typst content block `[...]`.
fn escape_content_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '[' | ']' | '#' | '@' | '*' | '_' | '$' | '<' | '>'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

// ---------------------------------------------------------------------------
// Typst world.
// ---------------------------------------------------------------------------

/// A repository-backed Typst world. `overlays` map a virtual path to generated
/// source so a transformed entrypoint and its transformed includes compile while
/// every other path (imports, `.yml`, assets) resolves against the real tree.
struct RenderWorld {
    root_dir: PathBuf,
    main: FileId,
    overlays: BTreeMap<String, String>,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl RenderWorld {
    fn new(
        root_dir: &Path,
        main_vpath: &str,
        overlays: BTreeMap<String, String>,
        edition: Edition,
    ) -> Self {
        let fonts = FontSearcher::new().include_system_fonts(false).search();
        let features = match edition {
            Edition::Html => Features::from_iter([Feature::Html]),
            Edition::Pdf => Features::default(),
        };

        Self {
            root_dir: root_dir.to_path_buf(),
            main: FileId::new(None, VirtualPath::new(main_vpath)),
            overlays,
            library: LazyHash::new(Library::builder().with_features(features).build()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts.iter().filter_map(|slot| slot.get()).collect(),
        }
    }

    fn overlay_key(id: FileId) -> String {
        id.vpath()
            .as_rootless_path()
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if let Some(path) = package_path::resolve(id)? {
            return Ok(path);
        }
        id.vpath()
            .resolve(&self.root_dir)
            .ok_or(FileError::AccessDenied)
    }
}

impl World for RenderWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if let Some(text) = self.overlays.get(&Self::overlay_key(id)) {
            return Ok(Source::new(id, text.clone()));
        }

        let path = self.resolve(id)?;
        if path.extension().is_some_and(|extension| extension != "typ") {
            return Err(FileError::NotSource);
        }
        let text = fs::read_to_string(&path).map_err(|source| FileError::from_io(source, &path))?;
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve(id)?;
        match fs::read(&path) {
            Ok(bytes) => Ok(Bytes::new(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound && is_image_id(id) => {
                Ok(Bytes::new(PLACEHOLDER_PNG.to_vec()))
            }
            Err(error) => Err(FileError::from_io(error, &path)),
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 5, 24)
    }
}

fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "gif" | "jpg" | "jpeg" | "png" | "svg" | "webp"
            )
        })
}

fn is_image_id(id: FileId) -> bool {
    is_image_path(id.vpath().as_rootless_path())
}

const PLACEHOLDER_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0, 5, 0, 1, 13, 10,
    45, 180, 0, 0, 0, 0, 73, 69, 68, 174, 66, 96, 130,
];

fn format_diagnostics<'a>(
    diagnostics: impl IntoIterator<Item = &'a SourceDiagnostic>,
) -> Vec<String> {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let mut message = diagnostic.message.to_string();
            for hint in &diagnostic.hints {
                write!(message, " hint: {hint}").unwrap();
            }
            message
        })
        .collect()
}
