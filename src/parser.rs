use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use typst_syntax::ast::{self, AstNode};
use typst_syntax::{SyntaxKind, SyntaxNode};

use crate::model::*;

mod calls;
mod markers;
mod projections;
mod world;

#[derive(Debug)]
pub enum ParseError {
    RootHasNoParent(PathBuf),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    NonUtf8Path(PathBuf),
    UnsupportedGlob {
        source_path: String,
        pattern: String,
    },
    Typst {
        source_path: String,
        diagnostics: Vec<String>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootHasNoParent(path) => {
                write!(f, "root path has no parent directory: {}", path.display())
            }
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::NonUtf8Path(path) => write!(f, "path is not valid UTF-8: {}", path.display()),
            Self::UnsupportedGlob {
                source_path,
                pattern,
            } => write!(
                f,
                "{source_path}: unsupported publisher.children glob pattern {pattern:?}"
            ),
            Self::Typst {
                source_path,
                diagnostics,
            } => write!(
                f,
                "{source_path}: Typst marker evaluation failed: {}",
                diagnostics.join("; ")
            ),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_publication(root_path: impl AsRef<Path>) -> Result<Publication, ParseError> {
    PublicationParser::new(root_path.as_ref())?.parse()
}

struct PublicationParser {
    root_dir: PathBuf,
    root_source_path: String,
    parsed_files: BTreeMap<String, ParsedFile>,
    reachable_order: Vec<String>,
    parents: BTreeMap<String, String>,
    support_files: BTreeSet<String>,
}

impl PublicationParser {
    fn new(root_path: &Path) -> Result<Self, ParseError> {
        let root_dir = root_path
            .parent()
            .ok_or_else(|| ParseError::RootHasNoParent(root_path.to_path_buf()))?
            .to_path_buf();
        let file_name = root_path
            .file_name()
            .ok_or_else(|| ParseError::RootHasNoParent(root_path.to_path_buf()))?;
        let root_source_path = file_name
            .to_str()
            .ok_or_else(|| ParseError::NonUtf8Path(root_path.to_path_buf()))?
            .to_string();

        Ok(Self {
            root_dir,
            root_source_path,
            parsed_files: BTreeMap::new(),
            reachable_order: Vec::new(),
            parents: BTreeMap::new(),
            support_files: BTreeSet::new(),
        })
    }

    fn parse(mut self) -> Result<Publication, ParseError> {
        let root = self.root_source_path.clone();
        self.visit(&root)?;

        let root_file = self.parsed_files.get(&self.root_source_path).unwrap();
        let root_node = self.node_from_file(root_file);
        let mut publication = Publication::new(root_node);

        for source_path in self.reachable_order.iter().skip(1) {
            let parsed = self.parsed_files.get(source_path).unwrap();
            publication.add_node(self.node_from_file(parsed));
        }

        for source_path in &self.reachable_order {
            let parsed = self.parsed_files.get(source_path).unwrap();
            add_source_path_property(&mut publication, parsed);
            if let Some(title) = &parsed.title {
                publication.add_property(Property::new(
                    property_id(&parsed.node_id, "title"),
                    parsed.node_id.clone(),
                    "title",
                    Value::String(title.text.clone()),
                    PropertySource::Heading { level: title.level },
                ));
            }
            for property in &parsed.properties {
                publication.add_property(property.clone());
            }
        }

        for source_path in &self.reachable_order {
            let parsed = self.parsed_files.get(source_path).unwrap();
            for scope in &parsed.scopes {
                publication.add_scope(scope.clone());
            }
        }

        projections::add_inherited_nav_projections(
            &mut publication,
            &self.parsed_files,
            &self.reachable_order,
        );

        for source_path in &self.reachable_order {
            let parsed = self.parsed_files.get(source_path).unwrap();
            for query in &parsed.queries {
                publication.add_query(query.clone());
            }
            for projection in &parsed.projections {
                publication.add_projection(projection.clone());
            }
        }

        for source_path in &self.reachable_order {
            let parsed = self.parsed_files.get(source_path).unwrap();
            for warning in &parsed.warnings {
                publication.add_parse_warning(warning.clone());
            }
        }

        for typ_file in self.all_typ_files()? {
            if !self.parsed_files.contains_key(&typ_file) {
                publication.add_parse_warning(ParseWarning::unreachable_typ_file(typ_file));
            }
        }

        Ok(publication)
    }

    fn visit(&mut self, source_path: &str) -> Result<(), ParseError> {
        if self.parsed_files.contains_key(source_path) {
            return Ok(());
        }

        let parsed = self.parse_file(source_path)?;
        let child_paths = parsed.child_paths.clone();
        self.reachable_order.push(source_path.to_string());
        self.parsed_files.insert(source_path.to_string(), parsed);

        for child_path in child_paths {
            self.parents
                .entry(child_path.clone())
                .or_insert_with(|| source_path.to_string());
            self.visit(&child_path)?;
        }

        Ok(())
    }

    fn parse_file(&mut self, source_path: &str) -> Result<ParsedFile, ParseError> {
        let full_path = self.root_dir.join(source_path);
        let text = fs::read_to_string(&full_path).map_err(|source| ParseError::Io {
            path: full_path.clone(),
            source,
        })?;
        let syntax = typst_syntax::parse(&text);

        let mut parsed = ParsedFile::new(source_path, text);
        self.extract_from_node(&syntax, &mut parsed)?;
        let evaluation = markers::evaluate_markers(self, source_path)?;
        self.support_files.extend(evaluation.support_files);
        for marker in evaluation.markers {
            calls::record_marker(self, &mut parsed, marker)?;
        }
        Ok(parsed)
    }

    fn node_from_file(&self, parsed: &ParsedFile) -> Node {
        let mut node = Node::new(parsed.node_id.clone(), parsed.source_path.clone())
            .with_authored_source(parsed.authored_source.clone())
            .with_children(parsed.child_paths.iter().map(|child| node_id(child)));
        node.parent = self
            .parents
            .get(&parsed.source_path)
            .map(|source_path| node_id(source_path));
        node
    }

    fn extract_from_node(
        &self,
        node: &SyntaxNode,
        parsed: &mut ParsedFile,
    ) -> Result<(), ParseError> {
        match node.kind() {
            SyntaxKind::Heading => {
                if let Some(heading) = node.cast::<ast::Heading>() {
                    parsed.record_heading(heading);
                }
            }
            SyntaxKind::Ref => {
                if let Some(reference) = node.cast::<ast::Ref>() {
                    parsed.record_citation(reference.target());
                }
            }
            _ => {}
        }

        for child in node.children() {
            self.extract_from_node(child, parsed)?;
        }

        Ok(())
    }

    fn expand_children_glob(
        &self,
        source_path: &str,
        pattern: &str,
    ) -> Result<Vec<String>, ParseError> {
        let Some(star_index) = pattern.find('*') else {
            return Ok(vec![normalize_source_path(pattern)]);
        };

        let slash_index = pattern[..star_index].rfind('/').map(|index| index + 1);
        let (directory, file_pattern) = match slash_index {
            Some(index) => (&pattern[..index], &pattern[index..]),
            None => ("", pattern),
        };
        let Some(file_star_index) = file_pattern.find('*') else {
            return Err(ParseError::UnsupportedGlob {
                source_path: source_path.to_string(),
                pattern: pattern.to_string(),
            });
        };
        if file_pattern[file_star_index + 1..].contains('*') || directory.contains('*') {
            return Err(ParseError::UnsupportedGlob {
                source_path: source_path.to_string(),
                pattern: pattern.to_string(),
            });
        }

        let prefix = &file_pattern[..file_star_index];
        let suffix = &file_pattern[file_star_index + 1..];
        let directory_path = self.root_dir.join(directory);
        let mut matches = Vec::new();

        for entry in fs::read_dir(&directory_path).map_err(|source| ParseError::Io {
            path: directory_path.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| ParseError::Io {
                path: directory_path.clone(),
                source,
            })?;
            let file_type = entry.file_type().map_err(|source| ParseError::Io {
                path: entry.path(),
                source,
            })?;
            if !file_type.is_file() {
                continue;
            }

            let file_name = entry
                .file_name()
                .into_string()
                .map_err(|_| ParseError::NonUtf8Path(entry.path()))?;
            if file_name.starts_with(prefix) && file_name.ends_with(suffix) {
                let rel_path = normalize_source_path(&format!("{directory}{file_name}"));
                if rel_path != source_path {
                    matches.push(rel_path);
                }
            }
        }

        matches.sort();
        Ok(matches)
    }

    fn all_typ_files(&self) -> Result<BTreeSet<String>, ParseError> {
        let mut files = BTreeSet::new();
        self.collect_typ_files("", &self.root_dir, &mut files)?;
        Ok(files)
    }

    fn collect_typ_files(
        &self,
        rel_dir: &str,
        directory: &Path,
        files: &mut BTreeSet<String>,
    ) -> Result<(), ParseError> {
        for entry in fs::read_dir(directory).map_err(|source| ParseError::Io {
            path: directory.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| ParseError::Io {
                path: directory.to_path_buf(),
                source,
            })?;
            let file_name = entry
                .file_name()
                .into_string()
                .map_err(|_| ParseError::NonUtf8Path(entry.path()))?;
            let rel_path = format!("{rel_dir}{file_name}");
            let file_type = entry.file_type().map_err(|source| ParseError::Io {
                path: entry.path(),
                source,
            })?;

            if file_type.is_dir() {
                self.collect_typ_files(&format!("{rel_path}/"), &entry.path(), files)?;
            } else if file_type.is_file()
                && rel_path.ends_with(".typ")
                && !self.support_files.contains(&rel_path)
                && !is_publisher_library_file(&rel_path)
            {
                files.insert(rel_path);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct ParsedFile {
    source_path: String,
    node_id: NodeId,
    authored_source: String,
    child_paths: Vec<String>,
    title: Option<Title>,
    scopes: Vec<Scope>,
    properties: Vec<Property>,
    queries: Vec<Query>,
    projections: Vec<Projection>,
    warnings: Vec<ParseWarning>,
    nav_suppressed: bool,
    declaration_order: usize,
    property_count: usize,
    projection_count: usize,
}

impl ParsedFile {
    fn new(source_path: &str, authored_source: String) -> Self {
        Self {
            source_path: source_path.to_string(),
            node_id: node_id(source_path),
            authored_source,
            child_paths: Vec::new(),
            title: None,
            scopes: Vec::new(),
            properties: Vec::new(),
            queries: Vec::new(),
            projections: Vec::new(),
            warnings: Vec::new(),
            nav_suppressed: false,
            declaration_order: 0,
            property_count: 0,
            projection_count: 0,
        }
    }

    fn record_heading(&mut self, heading: ast::Heading<'_>) {
        let level = heading.depth().get() as u8;
        let text = plain_markup(heading.body().to_untyped());

        match &self.title {
            None => {
                self.title = Some(Title { level, text });
            }
            Some(existing) if level < existing.level => {
                self.title = Some(Title { level, text });
            }
            Some(existing) if level == existing.level => {}
            Some(_) => {}
        }
    }

    fn record_citation(&mut self, target: &str) {
        if target != "cite" && target != "bib-ref" {
            return;
        }

        let id = self.next_property_id("citation");
        self.properties.push(Property::new(
            id,
            self.node_id.clone(),
            "citation",
            Value::TypstLabel(target.to_string()),
            PropertySource::RawTypst {
                expression: format!("@{target}"),
            },
        ));
    }

    pub(super) fn next_property_id(&mut self, key: &str) -> PropertyId {
        self.property_count += 1;
        PropertyId::new(format!(
            "prop:{}:{key}:{}",
            self.node_id, self.property_count
        ))
    }

    pub(super) fn next_query_id(&mut self, kind: &str) -> QueryId {
        QueryId::new(format!(
            "query:{}:{kind}:{}",
            self.node_id, self.projection_count
        ))
    }

    pub(super) fn next_projection_id(&mut self, kind: &str) -> ProjectionId {
        self.projection_count += 1;
        ProjectionId::new(format!(
            "projection:{}:{kind}:{}",
            self.node_id, self.projection_count
        ))
    }
}

#[derive(Clone, Debug)]
struct Title {
    level: u8,
    text: String,
}

fn add_source_path_property(publication: &mut Publication, parsed: &ParsedFile) {
    publication.add_property(Property::new(
        property_id(&parsed.node_id, "source-path"),
        parsed.node_id.clone(),
        "source-path",
        Value::String(parsed.source_path.clone()),
        PropertySource::Implicit {
            reason: "source path".to_string(),
        },
    ));
}

pub(super) fn plain_markup(node: &SyntaxNode) -> String {
    let mut text = String::new();
    collect_plain_text(node, &mut text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_plain_text(node: &SyntaxNode, out: &mut String) {
    match node.kind() {
        SyntaxKind::Text | SyntaxKind::Space => out.push_str(node.text()),
        _ => {
            for child in node.children() {
                collect_plain_text(child, out);
            }
        }
    }
}

fn node_id(source_path: &str) -> NodeId {
    NodeId::new(source_path.trim_end_matches(".typ"))
}

pub(super) fn property_id(node_id: &NodeId, key: &str) -> PropertyId {
    PropertyId::new(format!("prop:{node_id}:{key}"))
}

pub(super) fn unique_scope_id(kind: &str, node_id: &NodeId, scopes: &[Scope]) -> ScopeId {
    let base = format!("{kind}:{node_id}");
    if !scopes.iter().any(|scope| scope.id.as_str() == base) {
        return ScopeId::new(base);
    }

    let mut index = 2;
    loop {
        let candidate = format!("{kind}:{node_id}:{index}");
        if !scopes
            .iter()
            .any(|scope| scope.id.as_str() == candidate.as_str())
        {
            return ScopeId::new(candidate);
        }
        index += 1;
    }
}

pub(super) fn scope_kind(kind: &str) -> ScopeKind {
    match kind {
        "nav" => ScopeKind::Nav,
        "outline" => ScopeKind::Outline,
        "reference" => ScopeKind::Reference,
        "bibliography" => ScopeKind::Bibliography,
        other => ScopeKind::Other(other.to_string()),
    }
}

pub(super) fn normalize_source_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn is_publisher_library_file(path: &str) -> bool {
    matches!(path, "publisher.typ" | "publisher-nav.typ")
}
