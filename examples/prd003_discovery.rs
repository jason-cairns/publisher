use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime, Dict, Label, NativeElement, StyleChain, Value};
use typst::introspection::MetadataElem;
use typst::layout::PagedDocument;
use typst::model::HeadingElem;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::{LazyHash, PicoStr};
use typst::{Feature, Features, Library, LibraryExt, World};
use typst_kit::fonts::FontSearcher;

const FIXTURE_ROOT: &str = "examples/prd003_discovery_site";
const OUTPUT_ROOT: &str = "target/prd-003-discovery";
const MARKER_LABEL: &str = "publisher-marker";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = std::env::current_dir()?;
    let fixture_root = repo_root.join(FIXTURE_ROOT);
    let output_root = repo_root.join(OUTPUT_ROOT);
    fs::create_dir_all(&output_root)?;

    let graph = discover_graph(&repo_root)?;
    write_assemblies(&output_root, &graph.reachable)?;

    let source_results = graph
        .reachable
        .iter()
        .map(|source| compile_source_html(&repo_root, source))
        .collect::<Vec<_>>();
    let assembly_results = [
        (
            "full publication",
            compile_html(
                &repo_root,
                "target/prd-003-discovery/full-publication.typ",
                RenderMode::Normal,
            ),
        ),
        (
            "thesis scope",
            compile_html(
                &repo_root,
                "target/prd-003-discovery/scope-thesis.typ",
                RenderMode::Normal,
            ),
        ),
        (
            "writing plus thesis union",
            compile_html(
                &repo_root,
                "target/prd-003-discovery/scope-writing-thesis.typ",
                RenderMode::Normal,
            ),
        ),
    ];

    let mut report = String::new();
    writeln!(report, "PRD-003 discovery evidence")?;
    writeln!(report, "fixture: {}", fixture_root.display())?;
    writeln!(report)?;
    writeln!(report, "reachable source documents:")?;
    for (index, source) in graph.reachable.iter().enumerate() {
        writeln!(
            report,
            "- {}. {} -> {}",
            index + 1,
            source,
            Path::new(source).with_extension("html").display()
        )?;
    }
    writeln!(report)?;
    writeln!(report, "publish edges:")?;
    for (from, to) in &graph.edges {
        writeln!(report, "- {from} -> {to}")?;
    }
    writeln!(report)?;
    writeln!(report, "scopes:")?;
    for scope in &graph.scopes {
        writeln!(
            report,
            "- id={} title={} tags={} declared-in={}",
            scope.id,
            scope.title.as_deref().unwrap_or("<none>"),
            scope.tags.join(","),
            scope.source
        )?;
    }
    writeln!(report)?;
    writeln!(report, "duplicate titled scope warnings:")?;
    for warning in duplicate_title_warnings(&graph.scopes) {
        writeln!(report, "- {warning}")?;
    }
    writeln!(report)?;
    writeln!(report, "heading introspection per source:")?;
    for source in &graph.reachable {
        let doc = compile_paged(
            &repo_root,
            &format!("{FIXTURE_ROOT}/{source}"),
            RenderMode::SanitizeReferences,
        )?;
        let headings = headings(&doc);
        writeln!(report, "- {source}: {}", headings.join(" | "))?;
    }
    writeln!(report)?;
    writeln!(report, "source-local HTML compilation:")?;
    for result in &source_results {
        match result {
            Ok(path) => writeln!(report, "- ok: {}", path.display())?,
            Err(CompileFailure {
                source,
                diagnostics,
            }) => writeln!(report, "- failed: {source}: {}", diagnostics.join("; "))?,
        }
    }
    writeln!(report)?;
    writeln!(report, "generated assembly HTML compilation:")?;
    for (name, result) in &assembly_results {
        match result {
            Ok(path) => writeln!(report, "- ok: {name}: {}", path.display())?,
            Err(CompileFailure {
                source,
                diagnostics,
            }) => writeln!(
                report,
                "- failed: {name}: {source}: {}",
                diagnostics.join("; ")
            )?,
        }
    }
    writeln!(report)?;
    writeln!(report, "conclusions:")?;
    writeln!(
        report,
        "- metadata labeled <publisher-marker> is recoverable through PagedDocument.introspector.query(&Selector::Label(...))."
    )?;
    writeln!(
        report,
        "- ordinary outline and bibliography are naturally bounded by the compiled document or generated assembly."
    )?;
    writeln!(
        report,
        "- ordinary cross-source references require the referenced label to be present in the compiled document, so per-source HTML needs a reference/link adapter."
    )?;
    writeln!(
        report,
        "- publisher.in-scope cannot be a pure Typst context switch over omitted source documents; the publisher must assemble the selected region."
    )?;

    let evidence_path = output_root.join("evidence.txt");
    fs::write(&evidence_path, report)?;
    println!("{}", evidence_path.display());

    Ok(())
}

#[derive(Clone, Debug)]
struct Graph {
    reachable: Vec<String>,
    edges: Vec<(String, String)>,
    scopes: Vec<ScopeMarker>,
}

#[derive(Clone, Debug)]
struct ScopeMarker {
    source: String,
    id: String,
    title: Option<String>,
    tags: Vec<String>,
}

#[derive(Clone, Debug)]
enum Marker {
    Publish {
        path: String,
    },
    Scope {
        id: String,
        title: Option<String>,
        tags: Vec<String>,
    },
    Other,
}

fn discover_graph(repo_root: &Path) -> Result<Graph, Box<dyn std::error::Error>> {
    let mut graph = Graph {
        reachable: Vec::new(),
        edges: Vec::new(),
        scopes: Vec::new(),
    };
    let mut seen = BTreeSet::new();
    visit(repo_root, "index.typ", &mut seen, &mut graph)?;
    Ok(graph)
}

fn visit(
    repo_root: &Path,
    source: &str,
    seen: &mut BTreeSet<String>,
    graph: &mut Graph,
) -> Result<(), Box<dyn std::error::Error>> {
    if !seen.insert(source.to_string()) {
        return Ok(());
    }

    graph.reachable.push(source.to_string());
    let full_source = format!("{FIXTURE_ROOT}/{source}");
    let doc = compile_paged(repo_root, &full_source, RenderMode::SanitizeReferences)?;
    for marker in markers(&doc)? {
        match marker {
            Marker::Publish { path } => {
                graph.edges.push((source.to_string(), path.clone()));
                visit(repo_root, &path, seen, graph)?;
            }
            Marker::Scope { id, title, tags } => {
                graph.scopes.push(ScopeMarker {
                    source: source.to_string(),
                    id,
                    title,
                    tags,
                });
            }
            Marker::Other => {}
        }
    }
    Ok(())
}

fn write_assemblies(output_root: &Path, reachable: &[String]) -> io::Result<()> {
    let full = assembly_source(reachable);
    fs::write(output_root.join("full-publication.typ"), full)?;

    let thesis_sources = reachable
        .iter()
        .filter(|source| source.starts_with("thesis/"))
        .cloned()
        .collect::<Vec<_>>();
    fs::write(
        output_root.join("scope-thesis.typ"),
        assembly_source(&thesis_sources),
    )?;

    let writing_thesis_sources = reachable
        .iter()
        .filter(|source| source.starts_with("writing") || source.starts_with("thesis/"))
        .cloned()
        .collect::<Vec<_>>();
    fs::write(
        output_root.join("scope-writing-thesis.typ"),
        assembly_source(&writing_thesis_sources),
    )?;

    Ok(())
}

fn assembly_source(sources: &[String]) -> String {
    let mut out = String::new();
    writeln!(out, "#set heading(numbering: \"1.1\")").unwrap();
    for (index, source) in sources.iter().enumerate() {
        if index > 0 {
            writeln!(out).unwrap();
            writeln!(out, "#pagebreak()").unwrap();
        }
        writeln!(out, "#include \"../../{FIXTURE_ROOT}/{source}\"").unwrap();
    }
    out
}

fn duplicate_title_warnings(scopes: &[ScopeMarker]) -> Vec<String> {
    let mut by_title: BTreeMap<String, Vec<&ScopeMarker>> = BTreeMap::new();
    for scope in scopes {
        if let Some(title) = &scope.title {
            by_title.entry(title.clone()).or_default().push(scope);
        }
    }
    by_title
        .into_iter()
        .filter(|(_, scopes)| scopes.len() > 1)
        .map(|(title, scopes)| {
            let ids = scopes
                .into_iter()
                .map(|scope| scope.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("duplicate title {title:?} used by scope ids: {ids}")
        })
        .collect()
}

fn compile_source_html(repo_root: &Path, source: &str) -> Result<PathBuf, CompileFailure> {
    let full_source = format!("{FIXTURE_ROOT}/{source}");
    compile_html(repo_root, &full_source, RenderMode::Normal)
}

fn compile_html(
    repo_root: &Path,
    source: &str,
    mode: RenderMode,
) -> Result<PathBuf, CompileFailure> {
    let doc = compile_html_doc(repo_root, source, mode)?;
    let html = typst_html::html(&doc).map_err(|diagnostics| CompileFailure {
        source: source.to_string(),
        diagnostics: format_diagnostics(diagnostics.iter()),
    })?;
    let output_path = Path::new(OUTPUT_ROOT).join(Path::new(source).with_extension("html"));
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| CompileFailure {
            source: source.to_string(),
            diagnostics: vec![error.to_string()],
        })?;
    }
    fs::write(&output_path, html).map_err(|error| CompileFailure {
        source: source.to_string(),
        diagnostics: vec![error.to_string()],
    })?;
    Ok(output_path)
}

fn compile_paged(
    repo_root: &Path,
    source: &str,
    mode: RenderMode,
) -> Result<PagedDocument, CompileFailure> {
    let world = DiscoveryWorld::new(repo_root, source, mode, OutputKind::Paged);
    typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|diagnostics| CompileFailure {
            source: source.to_string(),
            diagnostics: format_diagnostics(diagnostics.iter()),
        })
}

fn compile_html_doc(
    repo_root: &Path,
    source: &str,
    mode: RenderMode,
) -> Result<typst_html::HtmlDocument, CompileFailure> {
    let world = DiscoveryWorld::new(repo_root, source, mode, OutputKind::Html);
    typst::compile::<typst_html::HtmlDocument>(&world)
        .output
        .map_err(|diagnostics| CompileFailure {
            source: source.to_string(),
            diagnostics: format_diagnostics(diagnostics.iter()),
        })
}

fn markers(doc: &PagedDocument) -> Result<Vec<Marker>, Box<dyn std::error::Error>> {
    let label = Label::new(PicoStr::intern(MARKER_LABEL)).expect("marker label is non-empty");
    let mut markers = Vec::new();
    for content in doc
        .introspector
        .query(&typst::foundations::Selector::Label(label))
    {
        let Some(metadata) = content.to_packed::<MetadataElem>() else {
            continue;
        };
        markers.push(decode_marker(&metadata.value)?);
    }
    Ok(markers)
}

fn decode_marker(value: &Value) -> Result<Marker, MarkerDecodeError> {
    let dict = expect_dict(value, "publisher marker value")?;
    let kind = required_string(dict, "kind")?;
    match kind.as_str() {
        "publish" => Ok(Marker::Publish {
            path: required_string(dict, "path")?,
        }),
        "scope" => Ok(Marker::Scope {
            id: required_string(dict, "id")?,
            title: optional_content_text(dict, "title")?,
            tags: string_array(dict, "tags")?,
        }),
        _ => Ok(Marker::Other),
    }
}

fn headings(doc: &PagedDocument) -> Vec<String> {
    doc.introspector
        .query(&HeadingElem::ELEM.select())
        .iter()
        .filter_map(|content| content.to_packed::<HeadingElem>())
        .map(|heading| {
            format!(
                "level={} text={}",
                heading.resolve_level(StyleChain::default()),
                heading.body.plain_text()
            )
        })
        .collect()
}

struct MarkerDecodeError {
    message: String,
}

impl MarkerDecodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Debug for MarkerDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Display for MarkerDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MarkerDecodeError {}

fn expect_dict<'a>(value: &'a Value, context: &str) -> Result<&'a Dict, MarkerDecodeError> {
    match value {
        Value::Dict(dict) => Ok(dict),
        other => Err(MarkerDecodeError::new(format!(
            "{context} must be a dictionary, got {}",
            other.ty().short_name()
        ))),
    }
}

fn field<'a>(dict: &'a Dict, key: &str) -> Option<&'a Value> {
    dict.get(key)
        .ok()
        .filter(|value| !matches!(value, Value::None))
}

fn required_string(dict: &Dict, key: &str) -> Result<String, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Str(value)) => Ok(value.to_string()),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be a string, got {}",
            other.ty().short_name()
        ))),
        None => Err(MarkerDecodeError::new(format!(
            "missing required field {key:?}"
        ))),
    }
}

fn optional_content_text(dict: &Dict, key: &str) -> Result<Option<String>, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Content(content)) => Ok(Some(content.plain_text().trim().to_string())),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be content, got {}",
            other.ty().short_name()
        ))),
        None => Ok(None),
    }
}

fn string_array(dict: &Dict, key: &str) -> Result<Vec<String>, MarkerDecodeError> {
    match field(dict, key) {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| match value {
                Value::Str(value) => Ok(value.to_string()),
                other => Err(MarkerDecodeError::new(format!(
                    "field {key:?} entries must be strings, got {}",
                    other.ty().short_name()
                ))),
            })
            .collect(),
        Some(other) => Err(MarkerDecodeError::new(format!(
            "field {key:?} must be an array, got {}",
            other.ty().short_name()
        ))),
        None => Ok(Vec::new()),
    }
}

#[derive(Clone, Copy)]
enum RenderMode {
    Normal,
    SanitizeReferences,
}

#[derive(Clone, Copy)]
enum OutputKind {
    Paged,
    Html,
}

struct DiscoveryWorld {
    root_dir: PathBuf,
    main: FileId,
    mode: RenderMode,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl DiscoveryWorld {
    fn new(root_dir: &Path, source_path: &str, mode: RenderMode, output: OutputKind) -> Self {
        let fonts = FontSearcher::new().include_system_fonts(false).search();
        let features = match output {
            OutputKind::Paged => Features::default(),
            OutputKind::Html => Features::from_iter([Feature::Html]),
        };

        Self {
            root_dir: root_dir.to_path_buf(),
            main: FileId::new(None, VirtualPath::new(source_path)),
            mode,
            library: LazyHash::new(Library::builder().with_features(features).build()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts.iter().filter_map(|slot| slot.get()).collect(),
        }
    }

    fn resolve(&self, id: FileId) -> FileResult<PathBuf> {
        if id.package().is_some() {
            return Err(FileError::Other(Some(
                "package imports are not supported in the PRD-003 discovery harness".into(),
            )));
        }

        id.vpath()
            .resolve(&self.root_dir)
            .ok_or_else(|| FileError::AccessDenied)
    }
}

impl World for DiscoveryWorld {
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

        let mut text =
            fs::read_to_string(&path).map_err(|error| FileError::from_io(error, &path))?;
        if matches!(self.mode, RenderMode::SanitizeReferences) && id == self.main {
            text = sanitize_references(&text);
        }
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = self.resolve(id)?;
        let bytes = fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 5, 31)
    }
}

fn sanitize_references(text: &str) -> String {
    let mut sanitized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '@' && chars.peek().is_some_and(|next| is_label_char(*next)) {
            while chars.peek().is_some_and(|next| is_label_char(*next)) {
                chars.next();
            }
            sanitized.push_str("[]");
        } else {
            sanitized.push(ch);
        }
    }

    sanitized
}

fn is_label_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '/')
}

struct CompileFailure {
    source: String,
    diagnostics: Vec<String>,
}

impl fmt::Debug for CompileFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.source, self.diagnostics.join("; "))
    }
}

impl fmt::Display for CompileFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.source, self.diagnostics.join("; "))
    }
}

impl std::error::Error for CompileFailure {}

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
