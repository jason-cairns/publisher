use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use typst_syntax::ast::{self, AstNode};
use typst_syntax::{SyntaxKind, SyntaxNode};

use crate::model::*;

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

        for source_path in &self.reachable_order {
            let parsed = self.parsed_files.get(source_path).unwrap();
            for query in &parsed.queries {
                publication.add_query(query.clone());
            }
            for projection in &parsed.projections {
                publication.add_projection(projection.clone());
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

    fn parse_file(&self, source_path: &str) -> Result<ParsedFile, ParseError> {
        let full_path = self.root_dir.join(source_path);
        let text = fs::read_to_string(&full_path).map_err(|source| ParseError::Io {
            path: full_path.clone(),
            source,
        })?;
        let syntax = typst_syntax::parse(&text);

        let mut parsed = ParsedFile::new(source_path);
        self.extract_from_node(&syntax, &mut parsed)?;
        Ok(parsed)
    }

    fn node_from_file(&self, parsed: &ParsedFile) -> Node {
        let mut node = Node::new(parsed.node_id.clone(), parsed.source_path.clone())
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
            SyntaxKind::FuncCall => {
                if let Some(call) = node.cast::<ast::FuncCall>() {
                    self.record_call(parsed, call)?;
                }
            }
            _ => {}
        }

        for child in node.children() {
            self.extract_from_node(child, parsed)?;
        }

        Ok(())
    }

    fn record_call(
        &self,
        parsed: &mut ParsedFile,
        call: ast::FuncCall<'_>,
    ) -> Result<(), ParseError> {
        let callee = raw_expr(call.callee());
        let args = CallArgs::from(call.args());

        match callee.as_str() {
            "publisher.child" => {
                if let Some(path) = args.first_string() {
                    parsed.child_paths.push(normalize_source_path(&path));
                }
            }
            "publisher.children" => {
                if let Some(pattern) = args.first_string() {
                    let children = self.expand_children_glob(&parsed.source_path, &pattern)?;
                    parsed.child_paths.extend(children);
                }
            }
            "publisher.scope" => {
                if let Some(kind) = args.named_string("kind") {
                    parsed.record_scope(&kind);
                }
            }
            "publisher.outline" => parsed.record_outline(args),
            "publisher.bibliography" => parsed.record_bibliography(args),
            "publisher.ref" => parsed.record_reference(args),
            "publisher.nav.suppress" | "nav.suppress" => parsed.record_nav_suppression(),
            _ if callee.starts_with("publisher.") => {
                parsed.warnings.push(ParseWarning {
                    source_path: Some(parsed.source_path.clone()),
                    kind: ParseWarningKind::UnsupportedPublisherCall,
                    message: format!("unsupported publisher call preserved for review: {callee}"),
                });
            }
            _ => {}
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
            } else if file_type.is_file() && rel_path.ends_with(".typ") {
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
    child_paths: Vec<String>,
    title: Option<Title>,
    scopes: Vec<Scope>,
    properties: Vec<Property>,
    queries: Vec<Query>,
    projections: Vec<Projection>,
    warnings: Vec<ParseWarning>,
    declaration_order: usize,
    property_count: usize,
    projection_count: usize,
}

impl ParsedFile {
    fn new(source_path: &str) -> Self {
        Self {
            source_path: source_path.to_string(),
            node_id: node_id(source_path),
            child_paths: Vec::new(),
            title: None,
            scopes: Vec::new(),
            properties: Vec::new(),
            queries: Vec::new(),
            projections: Vec::new(),
            warnings: Vec::new(),
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

    fn record_scope(&mut self, kind: &str) {
        let scope_kind = scope_kind(kind);
        let scope_id = unique_scope_id(kind, &self.node_id, &self.scopes);
        let scope = Scope::explicit(
            scope_id,
            scope_kind,
            self.node_id.clone(),
            self.declaration_order,
        )
        .with_name(kind);
        self.declaration_order += 1;
        self.scopes.push(scope);
    }

    fn record_outline(&mut self, args: CallArgs) {
        let query_id = self.next_query_id("outline");
        let projection_id = self.next_projection_id("outline");

        let mut query = Query::new(query_id.clone(), self.node_id.clone());
        query.selection = Authored::explicit(QuerySelection::Nodes);
        if let Some(target) = args.named_raw("target") {
            query.filters.push(QueryFilter {
                target: "target".to_string(),
                op: FilterOp::Raw("equivalent".to_string()),
                value: Value::RawTypst(target),
                source: ValueSource::Explicit,
            });
        }

        let mut projection = Projection::new(
            projection_id,
            self.node_id.clone(),
            ProjectionKind::Outline,
            query_id,
        );
        if let Some(depth) = args.named_i64("depth") {
            projection
                .rendering_attributes
                .push(Attribute::new("depth", Value::Number(depth)));
        }
        if let Some(title) = args.named_content_text("title") {
            projection
                .rendering_attributes
                .push(Attribute::new("title", Value::String(title)));
        }
        if let Some(target) = args.named_raw("target") {
            projection
                .rendering_attributes
                .push(Attribute::new("target", Value::RawTypst(target)));
        }

        self.queries.push(query);
        self.projections.push(projection);
    }

    fn record_bibliography(&mut self, args: CallArgs) {
        let query_id = self.next_query_id("bibliography");
        let projection_id = self.next_projection_id("bibliography");

        let mut query = Query::new(query_id.clone(), self.node_id.clone());
        query.selection = Authored::explicit(QuerySelection::Properties);
        if args.named_string("scope").as_deref() == Some("current-page") {
            query.search = Authored::explicit(QuerySearchRule::CurrentNode);
        }
        query.filters.push(QueryFilter {
            target: "property.key".to_string(),
            op: FilterOp::Equals,
            value: Value::String("citation".to_string()),
            source: ValueSource::Explicit,
        });

        let mut projection = Projection::new(
            projection_id,
            self.node_id.clone(),
            ProjectionKind::Bibliography,
            query_id,
        );
        if let Some(source) = args.first_string() {
            projection
                .rendering_attributes
                .push(Attribute::new("source", Value::String(source)));
        }
        if let Some(scope) = args.named_string("scope") {
            projection
                .rendering_attributes
                .push(Attribute::new("scope", Value::String(scope)));
        }

        self.queries.push(query);
        self.projections.push(projection);
    }

    fn record_reference(&mut self, args: CallArgs) {
        let Some(target) = args.first_label() else {
            return;
        };

        let query_id = self.next_query_id("reference");
        let projection_id = self.next_projection_id("reference");
        let mut query = Query::new(query_id.clone(), self.node_id.clone());
        query.selection = Authored::explicit(QuerySelection::ResolvedTarget);
        query.filters.push(QueryFilter {
            target: "target".to_string(),
            op: FilterOp::Equals,
            value: Value::TypstLabel(target.clone()),
            source: ValueSource::Explicit,
        });

        let mut projection = Projection::new(
            projection_id,
            self.node_id.clone(),
            ProjectionKind::Reference,
            query_id,
        );
        projection
            .rendering_attributes
            .push(Attribute::new("target", Value::TypstLabel(target)));

        self.queries.push(query);
        self.projections.push(projection);
    }

    fn record_nav_suppression(&mut self) {
        let property_id = property_id(&self.node_id, "nav-suppressed");
        self.properties.push(Property::new(
            property_id,
            self.node_id.clone(),
            "nav.suppressed",
            Value::Bool(true),
            PropertySource::ExplicitPublisherCall {
                call: "publisher.nav.suppress".to_string(),
            },
        ));

        let query_id = QueryId::new(format!("query:{}:nav", self.node_id));
        let projection_id = ProjectionId::new(format!("projection:{}:nav", self.node_id));
        self.queries
            .push(Query::new(query_id.clone(), self.node_id.clone()));
        self.projections.push(Projection {
            id: projection_id,
            origin_node: self.node_id.clone(),
            kind: ProjectionKind::Navigation,
            query: query_id,
            rendering_attributes: Vec::new(),
            suppression: ProjectionSuppression::Suppressed {
                reason: "property(nav.suppressed)".to_string(),
            },
        });
    }

    fn next_property_id(&mut self, key: &str) -> PropertyId {
        self.property_count += 1;
        PropertyId::new(format!(
            "prop:{}:{key}:{}",
            self.node_id, self.property_count
        ))
    }

    fn next_query_id(&mut self, kind: &str) -> QueryId {
        QueryId::new(format!(
            "query:{}:{kind}:{}",
            self.node_id, self.projection_count
        ))
    }

    fn next_projection_id(&mut self, kind: &str) -> ProjectionId {
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

#[derive(Clone, Debug, Default)]
struct CallArgs {
    positional: Vec<ArgValue>,
    named: BTreeMap<String, ArgValue>,
}

impl<'a> From<ast::Args<'a>> for CallArgs {
    fn from(args: ast::Args<'a>) -> Self {
        let mut parsed = Self::default();
        for arg in args.items() {
            match arg {
                ast::Arg::Pos(expr) => parsed.positional.push(ArgValue::from_expr(expr)),
                ast::Arg::Named(named) => {
                    parsed.named.insert(
                        named.name().as_str().to_string(),
                        ArgValue::from_expr(named.expr()),
                    );
                }
                ast::Arg::Spread(spread) => parsed
                    .positional
                    .push(ArgValue::Raw(raw_expr(spread.expr()))),
            }
        }
        parsed
    }
}

impl CallArgs {
    fn first_string(&self) -> Option<String> {
        self.positional
            .iter()
            .find_map(ArgValue::as_string)
            .cloned()
    }

    fn first_label(&self) -> Option<String> {
        self.positional.iter().find_map(ArgValue::as_label).cloned()
    }

    fn named_string(&self, name: &str) -> Option<String> {
        self.named.get(name).and_then(ArgValue::as_string).cloned()
    }

    fn named_i64(&self, name: &str) -> Option<i64> {
        self.named.get(name).and_then(ArgValue::as_i64)
    }

    fn named_content_text(&self, name: &str) -> Option<String> {
        self.named
            .get(name)
            .and_then(ArgValue::as_content_text)
            .cloned()
    }

    fn named_raw(&self, name: &str) -> Option<String> {
        self.named.get(name).map(ArgValue::raw)
    }
}

#[derive(Clone, Debug)]
enum ArgValue {
    String(String),
    Number(i64),
    Label(String),
    ContentText(String),
    Raw(String),
}

impl ArgValue {
    fn from_expr(expr: ast::Expr<'_>) -> Self {
        match expr {
            ast::Expr::Str(value) => Self::String(value.get().to_string()),
            ast::Expr::Int(value) => Self::Number(value.get()),
            ast::Expr::Label(value) => Self::Label(value.get().to_string()),
            ast::Expr::ContentBlock(value) => {
                Self::ContentText(plain_markup(value.body().to_untyped()))
            }
            _ => Self::Raw(raw_expr(expr)),
        }
    }

    fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn as_label(&self) -> Option<&String> {
        match self {
            Self::Label(value) => Some(value),
            _ => None,
        }
    }

    fn as_content_text(&self) -> Option<&String> {
        match self {
            Self::ContentText(value) => Some(value),
            _ => None,
        }
    }

    fn raw(&self) -> String {
        match self {
            Self::String(value) => format!("{value:?}"),
            Self::Number(value) => value.to_string(),
            Self::Label(value) => format!("<{value}>"),
            Self::ContentText(value) => format!("[{value}]"),
            Self::Raw(value) => value.clone(),
        }
    }
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

fn raw_expr(expr: ast::Expr<'_>) -> String {
    expr.to_untyped().clone().into_text().to_string()
}

fn plain_markup(node: &SyntaxNode) -> String {
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

fn property_id(node_id: &NodeId, key: &str) -> PropertyId {
    PropertyId::new(format!("prop:{node_id}:{key}"))
}

fn unique_scope_id(kind: &str, node_id: &NodeId, scopes: &[Scope]) -> ScopeId {
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

fn scope_kind(kind: &str) -> ScopeKind {
    match kind {
        "nav" => ScopeKind::Nav,
        "outline" => ScopeKind::Outline,
        "reference" => ScopeKind::Reference,
        "bibliography" => ScopeKind::Bibliography,
        other => ScopeKind::Other(other.to_string()),
    }
}

fn normalize_source_path(path: &str) -> String {
    path.replace('\\', "/")
}
