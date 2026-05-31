use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{self, Display, Write};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::model::*;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    pub source_root: PathBuf,
    pub output_dir: PathBuf,
    pub artifact_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedArtifacts {
    pub typst_path: PathBuf,
    pub pdf_path: PathBuf,
    pub html_paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum RenderError {
    ValidationFailed { errors: Vec<ValidationError> },
    CreateOutputDir { path: PathBuf, source: io::Error },
    AssemblyWrite { path: PathBuf, source: io::Error },
    PagedCompilation { diagnostics: Vec<String> },
    PdfExport { diagnostics: Vec<String> },
    PdfWrite { path: PathBuf, source: io::Error },
    HtmlCompilation { diagnostics: Vec<String> },
    HtmlExport { diagnostics: Vec<String> },
    HtmlWrite { path: PathBuf, source: io::Error },
}

impl Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::ValidationFailed { errors } => {
                write!(f, "publication failed validation before render: {errors:?}")
            }
            RenderError::CreateOutputDir { path, source } => {
                write!(
                    f,
                    "failed to create render output directory {}: {source}",
                    path.display()
                )
            }
            RenderError::AssemblyWrite { path, source } => {
                write!(
                    f,
                    "failed to write Typst assembly {}: {source}",
                    path.display()
                )
            }
            RenderError::PagedCompilation { diagnostics } => {
                write!(
                    f,
                    "failed to compile paged Typst assembly: {}",
                    diagnostics.join("; ")
                )
            }
            RenderError::PdfExport { diagnostics } => {
                write!(f, "failed to export PDF: {}", diagnostics.join("; "))
            }
            RenderError::PdfWrite { path, source } => {
                write!(f, "failed to write PDF {}: {source}", path.display())
            }
            RenderError::HtmlCompilation { diagnostics } => {
                write!(
                    f,
                    "failed to compile HTML Typst assembly: {}",
                    diagnostics.join("; ")
                )
            }
            RenderError::HtmlExport { diagnostics } => {
                write!(f, "failed to export HTML: {}", diagnostics.join("; "))
            }
            RenderError::HtmlWrite { path, source } => {
                write!(f, "failed to write HTML {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for RenderError {}

pub fn render_publication(
    publication: &Publication,
    options: &RenderOptions,
) -> Result<RenderedArtifacts, RenderError> {
    let report = publication.validate();
    if !report.is_ok() {
        return Err(RenderError::ValidationFailed {
            errors: report.errors,
        });
    }

    fs::create_dir_all(&options.output_dir).map_err(|source| RenderError::CreateOutputDir {
        path: options.output_dir.clone(),
        source,
    })?;

    let typst_path = options
        .output_dir
        .join(format!("{}.typ", options.artifact_name));
    let pdf_path = options
        .output_dir
        .join(format!("{}.pdf", options.artifact_name));

    let render_sources = write_render_sources(publication, options)?;
    let assembly = render_pdf_source(&render_sources);
    fs::write(&typst_path, &assembly).map_err(|source| RenderError::AssemblyWrite {
        path: typst_path.clone(),
        source,
    })?;

    let world = AssemblyWorld::new(
        &options.output_dir,
        &options.source_root,
        typst_path
            .strip_prefix(&options.output_dir)
            .ok()
            .and_then(|path| path.to_str())
            .unwrap_or("api-sketch.typ"),
    );

    let paged = typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|diagnostics| RenderError::PagedCompilation {
            diagnostics: format_diagnostics(diagnostics.iter()),
        })?;
    let pdf = typst_pdf::pdf(&paged, &PdfOptions::default()).map_err(|diagnostics| {
        RenderError::PdfExport {
            diagnostics: format_diagnostics(diagnostics.iter()),
        }
    })?;
    fs::write(&pdf_path, pdf).map_err(|source| RenderError::PdfWrite {
        path: pdf_path.clone(),
        source,
    })?;

    let stale_single_html = options
        .output_dir
        .join(format!("{}.html", options.artifact_name));
    if stale_single_html.exists() {
        fs::remove_file(&stale_single_html).map_err(|source| RenderError::HtmlWrite {
            path: stale_single_html.clone(),
            source,
        })?;
    }

    let html_paths = render_html_pages(&render_sources, options)?;

    Ok(RenderedArtifacts {
        typst_path,
        pdf_path,
        html_paths,
    })
}

fn render_html_pages(
    render_sources: &[RenderSource],
    options: &RenderOptions,
) -> Result<Vec<PathBuf>, RenderError> {
    let mut html_paths = Vec::new();

    for render_source in render_sources {
        if let Some(parent) = render_source.html_path.parent() {
            fs::create_dir_all(parent).map_err(|source| RenderError::CreateOutputDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let world = AssemblyWorld::new(
            &options.output_dir,
            &options.source_root,
            &render_source.html_virtual_path,
        );
        let html_document = typst::compile::<typst_html::HtmlDocument>(&world)
            .output
            .map_err(|diagnostics| RenderError::HtmlCompilation {
                diagnostics: format_diagnostics(diagnostics.iter()),
            })?;
        let html =
            typst_html::html(&html_document).map_err(|diagnostics| RenderError::HtmlExport {
                diagnostics: format_diagnostics(diagnostics.iter()),
            })?;
        fs::write(&render_source.html_path, html).map_err(|source| RenderError::HtmlWrite {
            path: render_source.html_path.clone(),
            source,
        })?;
        html_paths.push(render_source.html_path.clone());
    }

    Ok(html_paths)
}

fn html_path_for_route(output_dir: &Path, html_route: &str) -> PathBuf {
    output_dir.join(html_route)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RenderSource {
    html_virtual_path: String,
    pdf_virtual_path: String,
    html_path: PathBuf,
}

fn write_render_sources(
    publication: &Publication,
    options: &RenderOptions,
) -> Result<Vec<RenderSource>, RenderError> {
    let typst_dir = options.output_dir.join("typst");
    fs::create_dir_all(&typst_dir).map_err(|source| RenderError::CreateOutputDir {
        path: typst_dir.clone(),
        source,
    })?;

    let mut render_sources = Vec::new();
    for node in &publication.nodes {
        let html_virtual_path = generated_html_typst_path(&node.source_path);
        write_generated_node_source(
            publication,
            node,
            RenderBoundary::HtmlSource,
            &options.output_dir,
            &html_virtual_path,
        )?;

        let pdf_virtual_path = generated_pdf_typst_path(&node.source_path);
        write_generated_node_source(
            publication,
            node,
            RenderBoundary::CombinedPdf,
            &options.output_dir,
            &pdf_virtual_path,
        )?;

        render_sources.push(RenderSource {
            html_virtual_path,
            pdf_virtual_path,
            html_path: html_path_for_route(&options.output_dir, &node.html_route),
        });
    }

    Ok(render_sources)
}

fn write_generated_node_source(
    publication: &Publication,
    node: &Node,
    boundary: RenderBoundary,
    output_dir: &Path,
    virtual_path: &str,
) -> Result<(), RenderError> {
    let typst_path = output_dir.join(virtual_path);
    if let Some(parent) = typst_path.parent() {
        fs::create_dir_all(parent).map_err(|source| RenderError::CreateOutputDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let source = renderable_node_source(publication, node, boundary);
    fs::write(&typst_path, source).map_err(|source| RenderError::AssemblyWrite {
        path: typst_path,
        source,
    })
}

fn generated_html_typst_path(source_path: &str) -> String {
    format!("typst/{source_path}")
}

fn generated_pdf_typst_path(source_path: &str) -> String {
    format!("typst-pdf/{source_path}")
}

fn render_pdf_source(render_sources: &[RenderSource]) -> String {
    let mut source = String::new();
    for (index, render_source) in render_sources.iter().enumerate() {
        if index > 0 {
            writeln!(source, "#pagebreak()").unwrap();
            writeln!(source).unwrap();
        }
        writeln!(source, "#include \"{}\"", render_source.pdf_virtual_path).unwrap();
    }
    source
}

struct AssemblyWorld {
    main: FileId,
    root_dir: PathBuf,
    fallback_root_dir: PathBuf,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl AssemblyWorld {
    fn new(root_dir: &Path, fallback_root_dir: &Path, source_path: &str) -> Self {
        let fonts = FontSearcher::new().include_system_fonts(false).search();

        Self {
            main: FileId::new(None, VirtualPath::new(source_path)),
            root_dir: root_dir.to_path_buf(),
            fallback_root_dir: fallback_root_dir.to_path_buf(),
            library: LazyHash::new(
                Library::builder()
                    .with_features(Features::from_iter([Feature::Html]))
                    .build(),
            ),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts.iter().filter_map(|slot| slot.get()).collect(),
        }
    }

    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if id.package().is_some() {
            return Err(FileError::Other(Some(
                "package imports are not supported during publication rendering".into(),
            )));
        }

        let relative = id.vpath().as_rootless_path().to_path_buf();
        let generated = self.root_dir.join(&relative);
        if generated.exists() {
            return Ok(generated);
        }

        let fallback = self.fallback_root_dir.join(&relative);
        if fallback.exists() {
            return Ok(fallback);
        }

        if let Ok(original_relative) = relative.strip_prefix("typst") {
            let fallback = self.fallback_root_dir.join(original_relative);
            if fallback.exists() {
                return Ok(fallback);
            }
        }

        if let Ok(original_relative) = relative.strip_prefix("typst-pdf") {
            let fallback = self.fallback_root_dir.join(original_relative);
            if fallback.exists() {
                return Ok(fallback);
            }
        }

        Ok(generated)
    }
}

impl World for AssemblyWorld {
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
        let path = self.resolve(id)?;
        if path.extension().is_some_and(|extension| extension != "typ") {
            return Err(FileError::NotSource);
        }

        let text = fs::read_to_string(&path).map_err(|source| file_error(source, &path))?;
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if is_image_id(id) {
            return Ok(Bytes::new(PLACEHOLDER_PNG.to_vec()));
        }

        let path = self.resolve(id)?;
        let bytes = fs::read(&path).map_err(|source| file_error(source, &path))?;
        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 5, 24)
    }
}

fn file_error(error: io::Error, path: &Path) -> FileError {
    FileError::from_io(error, path)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyDocument {
    pub title: String,
    pub nodes: Vec<AssemblyNode>,
}

impl AssemblyDocument {
    pub fn to_typst(&self) -> String {
        let mut out = String::new();
        writeln!(out, "= {}", self.title).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "Generated from the typed publication model. Query evaluation is not implemented in this assembly.").unwrap();

        for node in &self.nodes {
            writeln!(out).unwrap();
            writeln!(
                out,
                "== {}",
                node.title.as_deref().unwrap_or(&node.source_path)
            )
            .unwrap();
            writeln!(out).unwrap();
            writeln!(out, "Source: `{}`", node.source_path).unwrap();
            writeln!(out, "Node id: `{}`", node.id).unwrap();
            writeln!(
                out,
                "Parent: `{}`",
                node.parent.as_deref().unwrap_or("<root>")
            )
            .unwrap();
            writeln!(
                out,
                "Children: {}",
                if node.children.is_empty() {
                    "`<none>`".to_string()
                } else {
                    node.children
                        .iter()
                        .map(|child| format!("`{child}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )
            .unwrap();

            writeln!(out).unwrap();
            writeln!(out, "Property summary:").unwrap();
            if node.properties.is_empty() {
                writeln!(out, "- `<none>`").unwrap();
            } else {
                for property in &node.properties {
                    writeln!(out, "- `{property}`").unwrap();
                }
            }

            writeln!(out).unwrap();
            writeln!(out, "Authored source:").unwrap();
            writeln!(out, "{}", fenced_code_block("typ", &node.authored_source)).unwrap();

            writeln!(out).unwrap();
            writeln!(out, "Projection placeholders:").unwrap();
            if node.projections.is_empty() {
                writeln!(out, "- `<none>`").unwrap();
            } else {
                for projection in &node.projections {
                    writeln!(out).unwrap();
                    writeln!(out, "{}", projection.to_typst_placeholder()).unwrap();
                }
            }
        }

        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyNode {
    pub id: String,
    pub source_path: String,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub title: Option<String>,
    pub properties: Vec<String>,
    pub authored_source: String,
    pub projections: Vec<AssemblyProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyProjection {
    pub id: String,
    pub kind: String,
    pub origin: String,
    pub rendering_attributes: Vec<String>,
    pub suppression: String,
    pub query: AssemblyQuery,
}

impl AssemblyProjection {
    fn to_typst_placeholder(&self) -> String {
        let mut detail = String::new();
        writeln!(detail, "projection-placeholder").unwrap();
        writeln!(detail, "id: {}", self.id).unwrap();
        writeln!(detail, "kind: {}", self.kind).unwrap();
        writeln!(detail, "origin: {}", self.origin).unwrap();
        writeln!(detail, "suppression: {}", self.suppression).unwrap();
        writeln!(
            detail,
            "rendering: {}",
            if self.rendering_attributes.is_empty() {
                "<none>".to_string()
            } else {
                self.rendering_attributes.join(", ")
            }
        )
        .unwrap();
        write!(detail, "{}", self.query.to_detail()).unwrap();

        format!(
            "#block(stroke: gray, inset: 8pt)[\n*Projection placeholder*\n{}\n]",
            fenced_code_block("text", detail.trim_end())
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyQuery {
    pub id: String,
    pub origin: String,
    pub selection: String,
    pub search: String,
    pub filters: Vec<String>,
    pub ordering: String,
    pub visibility: String,
    pub fallback: String,
    pub ambiguity: String,
}

impl AssemblyQuery {
    fn to_detail(&self) -> String {
        let mut detail = String::new();
        writeln!(detail, "query: {}", self.id).unwrap();
        writeln!(detail, "query-origin: {}", self.origin).unwrap();
        writeln!(detail, "query-selection: {}", self.selection).unwrap();
        writeln!(detail, "query-search: {}", self.search).unwrap();
        if self.filters.is_empty() {
            writeln!(detail, "query-filters: <none>").unwrap();
        } else {
            for filter in &self.filters {
                writeln!(detail, "query-filter: {filter}").unwrap();
            }
        }
        writeln!(detail, "query-ordering: {}", self.ordering).unwrap();
        writeln!(detail, "query-visibility: {}", self.visibility).unwrap();
        writeln!(detail, "query-fallback: {}", self.fallback).unwrap();
        writeln!(detail, "query-ambiguity: {}", self.ambiguity).unwrap();
        detail
    }
}

pub fn build_assembly(publication: &Publication) -> AssemblyDocument {
    AssemblyDocument {
        title: "API Sketch Publication".to_string(),
        nodes: publication
            .nodes
            .iter()
            .map(|node| build_assembly_node(publication, node))
            .collect(),
    }
}

pub fn render_assembly(publication: &Publication) -> String {
    build_assembly(publication).to_typst()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RenderBoundary {
    HtmlSource,
    CombinedPdf,
}

fn renderable_node_source(
    publication: &Publication,
    node: &Node,
    boundary: RenderBoundary,
) -> String {
    let mut placeholders = ProjectionPlaceholders::new(publication, node);
    let mut source =
        replace_publisher_projection_calls(&node.authored_source.typst, &mut placeholders);
    source = lower_scope_local_outlines(publication, node, &source);
    source = lower_bibliographies(publication, node, boundary, &source);
    let inherited_navigation = placeholders.remaining_navigation_placeholders();
    if !inherited_navigation.is_empty() {
        source = format!("{inherited_navigation}\n\n{source}");
    }
    adapt_reference_shorthand(publication, node, boundary, &source)
}

fn lower_bibliographies(
    publication: &Publication,
    node: &Node,
    boundary: RenderBoundary,
    source: &str,
) -> String {
    if boundary == RenderBoundary::HtmlSource {
        return source.to_string();
    }

    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    let mut replaced = 0usize;

    while index < source.len() {
        if let Some(open_paren) = bibliography_call_at(source, index) {
            if let Some(end) = find_call_end(source, open_paren) {
                replaced += 1;
                out.push_str(&combined_render_bibliography_placeholder(
                    publication,
                    node,
                    replaced,
                ));
                index = end;
                continue;
            }
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

fn bibliography_call_at(source: &str, index: usize) -> Option<usize> {
    let name = "#bibliography";
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

    if source[open_paren..].starts_with('(') {
        Some(open_paren)
    } else {
        None
    }
}

fn combined_render_bibliography_placeholder(
    publication: &Publication,
    node: &Node,
    bibliography_index: usize,
) -> String {
    let bibliography_sources = bibliography_sources_for_node(publication, node);
    let source = bibliography_sources
        .get(bibliography_index.saturating_sub(1))
        .map(String::as_str)
        .unwrap_or("<unknown>");
    let scope = nearest_publication_scope(publication, node)
        .map(|scope| scope.id.to_string())
        .unwrap_or_else(|| "<none>".to_string());

    let mut detail = String::new();
    writeln!(detail, "source: {source}").unwrap();
    writeln!(detail, "scope: {scope}").unwrap();
    writeln!(detail, "render-boundary: combined-pdf").unwrap();
    writeln!(
        detail,
        "note: ordinary bibliography call preserved in source-local HTML input"
    )
    .unwrap();

    format!(
        "#block(stroke: gray, inset: 8pt)[\n*Scoped bibliography*\n{}\n]",
        fenced_code_block("text", detail.trim_end())
    )
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

fn lower_scope_local_outlines(publication: &Publication, node: &Node, source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0;

    while index < source.len() {
        if let Some(open_paren) = outline_call_at(source, index) {
            if let Some(end) = find_call_end(source, open_paren) {
                let target = source[open_paren + 1..end - 1].trim();
                out.push_str(&scope_local_outline_block(publication, node, target));
                index = end;
                continue;
            }
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

fn outline_call_at(source: &str, index: usize) -> Option<usize> {
    let name = "#outline";
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

    if source[open_paren..].starts_with('(') {
        Some(open_paren)
    } else {
        None
    }
}

fn scope_local_outline_block(publication: &Publication, node: &Node, target: &str) -> String {
    let Some(scope) = nearest_publication_scope(publication, node) else {
        return format!(
            "#block(stroke: gray, inset: 8pt)[\n*Scope-local outline*\n{}\n]",
            fenced_code_block("text", "scope: <none>\nheadings: <none>")
        );
    };

    let mut detail = String::new();
    writeln!(detail, "scope: {}", scope.id).unwrap();
    if !target.is_empty() {
        writeln!(detail, "target: {target}").unwrap();
    }
    writeln!(detail, "headings:").unwrap();

    let headings = scope_outline_headings(publication, scope);
    if headings.is_empty() {
        writeln!(detail, "- <none>").unwrap();
    } else {
        for heading in headings {
            writeln!(detail, "- {heading}").unwrap();
        }
    }

    format!(
        "#block(stroke: gray, inset: 8pt)[\n*Scope-local outline*\n{}\n]",
        fenced_code_block("text", detail.trim_end())
    )
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

fn scope_outline_headings(publication: &Publication, scope: &Scope) -> Vec<String> {
    publication
        .nodes
        .iter()
        .filter(|candidate| scope.covers(publication, &candidate.id))
        .filter_map(|candidate| title_for_node(publication, &candidate.id))
        .collect()
}

struct ProjectionPlaceholders {
    outlines: VecDeque<AssemblyProjection>,
    bibliographies: VecDeque<AssemblyProjection>,
    references: VecDeque<AssemblyProjection>,
    navigation: VecDeque<AssemblyProjection>,
    suppressed_navigation: VecDeque<AssemblyProjection>,
}

impl ProjectionPlaceholders {
    fn new(publication: &Publication, node: &Node) -> Self {
        let projections = node
            .projections
            .iter()
            .filter_map(|projection_id| publication.projection(projection_id));

        let mut placeholders = Self {
            outlines: VecDeque::new(),
            bibliographies: VecDeque::new(),
            references: VecDeque::new(),
            navigation: VecDeque::new(),
            suppressed_navigation: VecDeque::new(),
        };

        for projection in projections {
            let placeholder = build_assembly_projection(publication, projection);
            match &projection.kind {
                ProjectionKind::Outline => placeholders.outlines.push_back(placeholder),
                ProjectionKind::Bibliography => placeholders.bibliographies.push_back(placeholder),
                ProjectionKind::Reference => placeholders.references.push_back(placeholder),
                ProjectionKind::Navigation
                    if matches!(
                        projection.suppression,
                        ProjectionSuppression::Suppressed { .. }
                    ) =>
                {
                    placeholders.suppressed_navigation.push_back(placeholder);
                }
                ProjectionKind::Navigation => placeholders.navigation.push_back(placeholder),
                _ => {}
            }
        }

        placeholders
    }

    fn take(&mut self, call: PublisherProjectionCall) -> String {
        let projection = match call {
            PublisherProjectionCall::Outline => self.outlines.pop_front(),
            PublisherProjectionCall::Bibliography => self.bibliographies.pop_front(),
            PublisherProjectionCall::Reference => self.references.pop_front(),
            PublisherProjectionCall::NavSuppress => self.suppressed_navigation.pop_front(),
        };

        projection
            .map(|projection| projection.to_typst_placeholder())
            .unwrap_or_else(|| unmatched_projection_placeholder(call))
    }

    fn remaining_navigation_placeholders(&mut self) -> String {
        let mut source = String::new();
        while let Some(projection) = self.navigation.pop_front() {
            writeln!(source, "{}", projection.to_typst_placeholder()).unwrap();
            writeln!(source).unwrap();
        }
        while let Some(projection) = self.suppressed_navigation.pop_front() {
            writeln!(source, "{}", projection.to_typst_placeholder()).unwrap();
            writeln!(source).unwrap();
        }
        source.trim_end().to_string()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PublisherProjectionCall {
    Outline,
    Bibliography,
    Reference,
    NavSuppress,
}

fn replace_publisher_projection_calls(
    source: &str,
    placeholders: &mut ProjectionPlaceholders,
) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0;

    while index < source.len() {
        if let Some((call, open_paren)) = publisher_projection_call_at(source, index) {
            if let Some(end) = find_call_end(source, open_paren) {
                out.push_str(&placeholders.take(call));
                index = end;
                continue;
            }
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

fn publisher_projection_call_at(
    source: &str,
    index: usize,
) -> Option<(PublisherProjectionCall, usize)> {
    const CALLS: &[(&str, PublisherProjectionCall)] = &[
        (
            "#publisher.nav.suppress",
            PublisherProjectionCall::NavSuppress,
        ),
        (
            "#publisher.bibliography",
            PublisherProjectionCall::Bibliography,
        ),
        ("#publisher.outline", PublisherProjectionCall::Outline),
        ("#publisher.ref", PublisherProjectionCall::Reference),
    ];

    for (name, call) in CALLS {
        if !source[index..].starts_with(name) {
            continue;
        }

        let mut open_paren = index + name.len();
        while let Some(ch) = source[open_paren..].chars().next() {
            if !ch.is_whitespace() {
                break;
            }
            open_paren += ch.len_utf8();
        }

        if source[open_paren..].starts_with('(') {
            return Some((*call, open_paren));
        }
    }

    None
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct LabelTarget {
    node_id: NodeId,
    route: String,
    title: Option<String>,
}

fn adapt_reference_shorthand(
    publication: &Publication,
    node: &Node,
    boundary: RenderBoundary,
    source: &str,
) -> String {
    let labels = publication_label_targets(publication);
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
                if let Some(adapted) =
                    reference_replacement(publication, node, boundary, label, &labels)
                {
                    out.push_str(&adapted);
                } else {
                    write!(out, "#raw(\"@{label} placeholder\")").unwrap();
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

fn reference_replacement(
    publication: &Publication,
    origin: &Node,
    boundary: RenderBoundary,
    label: &str,
    labels: &BTreeMap<String, LabelTarget>,
) -> Option<String> {
    let target = labels.get(label)?;

    if boundary == RenderBoundary::HtmlSource && target.node_id == origin.id {
        return Some(format!("@{label}"));
    }

    match boundary {
        RenderBoundary::CombinedPdf => {
            let display = reference_display(publication, origin, target, label);
            Some(format!("#raw(\"{}\")", typst_string_text(&display)))
        }
        RenderBoundary::HtmlSource => {
            let route = relative_route(&origin.html_route, &target.route);
            let display = reference_display(publication, origin, target, label);
            Some(format!(
                "#link(\"{}#{}\")[#raw(\"{}\")]",
                typst_string_text(&route),
                typst_string_text(label),
                typst_string_text(&display)
            ))
        }
    }
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
    if node.kind() == SyntaxKind::Label {
        if let Some(label) = node.cast::<ast::Label>() {
            labels.push(label.get().to_string());
        }
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
    if let Some(scope_title) = cross_scope_reference_title(publication, origin, target) {
        if scope_title != title {
            return format!("{title}, {scope_title}");
        }
    }
    title
}

fn cross_scope_reference_title(
    publication: &Publication,
    origin: &Node,
    target: &LabelTarget,
) -> Option<String> {
    let origin_spine = publication.spine_for(&origin.id).ok()?;
    let target_spine = publication.spine_for(&target.node_id).ok()?;
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

fn typst_string_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn take_label(source: &str) -> Option<(&str, usize)> {
    let mut end = 0;
    for ch in source.chars() {
        if !is_label_char(ch) {
            break;
        }
        end += ch.len_utf8();
    }

    if end == 0 {
        None
    } else {
        Some((&source[..end], end))
    }
}

fn is_label_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '/')
}

fn unmatched_projection_placeholder(call: PublisherProjectionCall) -> String {
    let kind = match call {
        PublisherProjectionCall::Outline => "outline",
        PublisherProjectionCall::Bibliography => "bibliography",
        PublisherProjectionCall::Reference => "reference",
        PublisherProjectionCall::NavSuppress => "navigation-suppression",
    };

    format!(
        "#block(stroke: gray, inset: 8pt)[\n*Projection placeholder*\n```text\nprojection-placeholder\nkind: {kind}\nprojection-detail: unavailable\n```\n]"
    )
}

fn build_assembly_node(publication: &Publication, node: &Node) -> AssemblyNode {
    AssemblyNode {
        id: node.id.to_string(),
        source_path: node.source_path.clone(),
        parent: node
            .parent
            .as_ref()
            .map(|parent| source_path(publication, parent)),
        children: node
            .children
            .iter()
            .map(|child| source_path(publication, child))
            .collect(),
        title: title_for_node(publication, &node.id),
        properties: publication
            .properties_for_node(&node.id)
            .iter()
            .map(|property| format_property(property))
            .collect(),
        authored_source: node.authored_source.typst.clone(),
        projections: node
            .projections
            .iter()
            .filter_map(|projection_id| publication.projection(projection_id))
            .map(|projection| build_assembly_projection(publication, projection))
            .collect(),
    }
}

fn build_assembly_projection(
    publication: &Publication,
    projection: &Projection,
) -> AssemblyProjection {
    AssemblyProjection {
        id: projection.id.to_string(),
        kind: projection_kind(&projection.kind),
        origin: source_path(publication, &projection.origin_node),
        rendering_attributes: projection
            .rendering_attributes
            .iter()
            .map(format_attribute)
            .collect(),
        suppression: projection_suppression(&projection.suppression),
        query: publication
            .query(&projection.query)
            .map(|query| build_assembly_query(publication, query))
            .unwrap_or_else(|| AssemblyQuery {
                id: format!("<missing {}>", projection.query),
                origin: projection.origin_node.to_string(),
                selection: "<missing>".to_string(),
                search: "<missing>".to_string(),
                filters: Vec::new(),
                ordering: "<missing>".to_string(),
                visibility: "<missing>".to_string(),
                fallback: "<missing>".to_string(),
                ambiguity: "<missing>".to_string(),
            }),
    }
}

fn build_assembly_query(publication: &Publication, query: &Query) -> AssemblyQuery {
    AssemblyQuery {
        id: query.id.to_string(),
        origin: source_path(publication, &query.origin_node),
        selection: format_sourced_value(&query.selection, query_selection),
        search: format_sourced_value(&query.search, query_search),
        filters: query.filters.iter().map(format_filter).collect(),
        ordering: format_sourced_value(&query.ordering, query_ordering),
        visibility: format_sourced_value(&query.visibility, query_visibility),
        fallback: format_sourced_value(&query.fallback, query_fallback),
        ambiguity: format_sourced_value(&query.ambiguity, query_ambiguity),
    }
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

fn source_path(publication: &Publication, node_id: &NodeId) -> String {
    publication
        .node(node_id)
        .map(|node| node.source_path.clone())
        .unwrap_or_else(|| node_id.to_string())
}

fn format_property(property: &Property) -> String {
    format!(
        "{} = {} {}",
        property.key,
        format_value(&property.value),
        property_source(&property.source)
    )
}

fn format_attribute(attribute: &Attribute) -> String {
    format!("{}={}", attribute.key, format_value(&attribute.value))
}

fn format_filter(filter: &QueryFilter) -> String {
    format!(
        "{} {} {} {}",
        filter.target,
        filter_op(&filter.op),
        format_value(&filter.value),
        value_source(&filter.source)
    )
}

fn format_sourced_value<T>(sourced_value: &SourcedValue<T>, format: fn(&T) -> String) -> String {
    format!(
        "{} {}",
        format(&sourced_value.value),
        value_source(&sourced_value.source)
    )
}

fn projection_suppression(suppression: &ProjectionSuppression) -> String {
    match suppression {
        ProjectionSuppression::NotSuppressed => "not-suppressed".to_string(),
        ProjectionSuppression::Suppressed { reason } => {
            format!("suppressed=true reason={reason}")
        }
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(value) => format!("{value:?}"),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::TypstLabel(value) => format!("<{value}>"),
        Value::RawTypst(value) => value.clone(),
    }
}

fn value_source(source: &ValueSource) -> &'static str {
    match source {
        ValueSource::Explicit => "explicit",
        ValueSource::Defaulted => "defaulted",
    }
}

fn property_source(source: &PropertySource) -> String {
    match source {
        PropertySource::Implicit { reason } => format!("implicit({reason})"),
        PropertySource::Heading { level } => format!("implicit(source-heading level={level})"),
        PropertySource::ExplicitPublisherCall { call } => format!("explicit({call})"),
        PropertySource::RawTypst { expression } => format!("explicit(raw {expression})"),
    }
}

fn projection_kind(kind: &ProjectionKind) -> String {
    match kind {
        ProjectionKind::Outline => "outline".to_string(),
        ProjectionKind::Navigation => "navigation".to_string(),
        ProjectionKind::Bibliography => "bibliography".to_string(),
        ProjectionKind::Reference => "reference".to_string(),
        ProjectionKind::Other(kind) => kind.clone(),
    }
}

fn query_selection(selection: &QuerySelection) -> String {
    match selection {
        QuerySelection::Nodes => "nodes".to_string(),
        QuerySelection::Scopes => "scopes".to_string(),
        QuerySelection::Properties => "properties".to_string(),
        QuerySelection::ResolvedTarget => "resolved-target".to_string(),
    }
}

fn query_search(search: &QuerySearchRule) -> String {
    match search {
        QuerySearchRule::CurrentNode => "current-node".to_string(),
        QuerySearchRule::CurrentScope => "current-scope".to_string(),
        QuerySearchRule::NearestScope => "nearest-scope".to_string(),
        QuerySearchRule::ActiveScopes { order } => format!("active-scopes({})", scope_order(order)),
        QuerySearchRule::NamedScope(name) => format!("named-scope({name})"),
        QuerySearchRule::WholePublication => "whole-publication".to_string(),
    }
}

fn scope_order(order: &ScopeSearchOrder) -> &'static str {
    match order {
        ScopeSearchOrder::NearestToGlobal => "nearest-to-global",
        ScopeSearchOrder::GlobalToNearest => "global-to-nearest",
    }
}

fn query_ordering(ordering: &QueryOrdering) -> String {
    match ordering {
        QueryOrdering::PublicationTree => "publication-tree".to_string(),
        QueryOrdering::SourceOrder => "source-order".to_string(),
        QueryOrdering::Raw(value) => value.clone(),
    }
}

fn query_visibility(visibility: &QueryVisibility) -> String {
    match visibility {
        QueryVisibility::VisibleFromOrigin => "visible-from-origin".to_string(),
        QueryVisibility::IncludeHidden => "include-hidden".to_string(),
    }
}

fn query_fallback(fallback: &QueryFallback) -> String {
    match fallback {
        QueryFallback::Empty => "empty".to_string(),
        QueryFallback::Error => "error".to_string(),
        QueryFallback::Raw(value) => value.clone(),
    }
}

fn query_ambiguity(ambiguity: &QueryAmbiguity) -> String {
    match ambiguity {
        QueryAmbiguity::AllowMany => "allow-many".to_string(),
        QueryAmbiguity::Error => "error".to_string(),
        QueryAmbiguity::First => "first".to_string(),
    }
}

fn filter_op(op: &FilterOp) -> String {
    match op {
        FilterOp::Equals => "==".to_string(),
        FilterOp::Contains => "contains".to_string(),
        FilterOp::Raw(value) => value.clone(),
    }
}

fn fenced_code_block(info: &str, content: &str) -> String {
    let fence = fence_for(content);
    let mut out = String::new();
    writeln!(out, "{fence}{info}").unwrap();
    write!(out, "{content}").unwrap();
    if !content.ends_with('\n') {
        writeln!(out).unwrap();
    }
    write!(out, "{fence}").unwrap();
    out
}

fn fence_for(content: &str) -> String {
    let mut longest = 0;
    let mut current = 0;
    for character in content.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    "`".repeat(longest.max(2) + 1)
}
