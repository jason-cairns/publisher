use std::collections::HashSet;
use std::fmt;

use crate::validation::{ValidationError, ValidationReport};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NodeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(String);

impl ScopeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ScopeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ScopeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyId(String);

impl PropertyId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PropertyId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PropertyId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QueryId(String);

impl QueryId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for QueryId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for QueryId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for QueryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectionId(String);

impl ProjectionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ProjectionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ProjectionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ProjectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Publication {
    pub root_node: NodeId,
    pub nodes: Vec<Node>,
    pub scopes: Vec<Scope>,
    pub properties: Vec<Property>,
    pub queries: Vec<Query>,
    pub projections: Vec<Projection>,
    pub parse_warnings: Vec<ParseWarning>,
}

impl Publication {
    pub fn new(root: Node) -> Self {
        let root_node = root.id.clone();
        let mut publication = Self {
            root_node: root_node.clone(),
            nodes: vec![root],
            scopes: vec![Scope::global(root_node.clone())],
            properties: Vec::new(),
            queries: Vec::new(),
            projections: Vec::new(),
            parse_warnings: Vec::new(),
        };
        publication.ensure_current_page_scope(&root_node);
        publication
    }

    pub fn implicit_global_scope_id() -> ScopeId {
        ScopeId::new("global")
    }

    pub fn implicit_current_page_scope_id(node_id: &NodeId) -> ScopeId {
        ScopeId::new(format!("current-page:{node_id}"))
    }

    pub fn add_node(&mut self, node: Node) {
        let node_id = node.id.clone();
        self.nodes.push(node);
        self.ensure_current_page_scope(&node_id);
    }

    pub fn add_scope(&mut self, scope: Scope) {
        if let Some(node) = self.node_mut(&scope.root_node) {
            if !scope.implicit && !node.explicit_scopes.contains(&scope.id) {
                node.explicit_scopes.push(scope.id.clone());
            }
        }
        self.scopes.push(scope);
    }

    pub fn add_property(&mut self, property: Property) {
        if let Some(node) = self.node_mut(&property.owning_node) {
            if !node.properties.contains(&property.id) {
                node.properties.push(property.id.clone());
            }
        }
        self.properties.push(property);
    }

    pub fn add_query(&mut self, query: Query) {
        self.queries.push(query);
    }

    pub fn add_projection(&mut self, projection: Projection) {
        if let Some(node) = self.node_mut(&projection.origin_node) {
            if !node.projections.contains(&projection.id) {
                node.projections.push(projection.id.clone());
            }
        }
        self.projections.push(projection);
    }

    pub fn add_parse_warning(&mut self, warning: ParseWarning) {
        self.parse_warnings.push(warning);
    }

    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|node| &node.id == id)
    }

    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|node| &node.id == id)
    }

    pub fn scope(&self, id: &ScopeId) -> Option<&Scope> {
        self.scopes.iter().find(|scope| &scope.id == id)
    }

    pub fn property(&self, id: &PropertyId) -> Option<&Property> {
        self.properties.iter().find(|property| &property.id == id)
    }

    pub fn query(&self, id: &QueryId) -> Option<&Query> {
        self.queries.iter().find(|query| &query.id == id)
    }

    pub fn projection(&self, id: &ProjectionId) -> Option<&Projection> {
        self.projections
            .iter()
            .find(|projection| &projection.id == id)
    }

    pub fn properties_for_node(&self, node_id: &NodeId) -> Vec<&Property> {
        self.properties
            .iter()
            .filter(|property| &property.owning_node == node_id)
            .collect()
    }

    pub fn property_scope_context(
        &self,
        property_id: &PropertyId,
    ) -> Result<Spine, ValidationError> {
        let property =
            self.property(property_id)
                .ok_or_else(|| ValidationError::MissingPropertyOwner {
                    property_id: property_id.clone(),
                    owner: NodeId::new("<unknown>"),
                })?;
        self.spine_for(&property.owning_node)
    }

    pub fn spine_for(&self, node_id: &NodeId) -> Result<Spine, ValidationError> {
        if self.node(node_id).is_none() {
            return Err(ValidationError::MissingNode {
                node_id: node_id.clone(),
            });
        }

        let mut active_scope_ids = Vec::new();
        let global_scope = self
            .scopes
            .iter()
            .find(|scope| scope.kind == ScopeKind::Global && scope.root_node == self.root_node)
            .ok_or_else(|| ValidationError::MissingScope {
                scope_id: Self::implicit_global_scope_id(),
            })?;
        active_scope_ids.push(global_scope.id.clone());

        for ancestor_id in self.ancestor_chain(node_id) {
            if &ancestor_id == node_id {
                break;
            }
            for scope in self.explicit_scopes_declared_by(&ancestor_id) {
                if scope.covers(self, node_id) {
                    active_scope_ids.push(scope.id.clone());
                }
            }
        }

        let current_page_id = Self::implicit_current_page_scope_id(node_id);
        if self.scope(&current_page_id).is_none() {
            return Err(ValidationError::MissingScope {
                scope_id: current_page_id,
            });
        }
        active_scope_ids.push(current_page_id);

        for scope in self.explicit_scopes_declared_by(node_id) {
            if scope.covers(self, node_id) {
                active_scope_ids.push(scope.id.clone());
            }
        }

        Ok(Spine {
            node: node_id.clone(),
            scopes: active_scope_ids,
        })
    }

    pub fn validate(&self) -> ValidationReport {
        crate::validation::validate_publication(self)
    }

    fn ensure_current_page_scope(&mut self, node_id: &NodeId) {
        let current_page_id = Self::implicit_current_page_scope_id(node_id);
        if self.scope(&current_page_id).is_none() {
            self.scopes.push(Scope::current_page(node_id.clone()));
        }
    }

    fn explicit_scopes_declared_by(&self, node_id: &NodeId) -> Vec<&Scope> {
        let mut scopes: Vec<_> = self
            .scopes
            .iter()
            .filter(|scope| !scope.implicit && &scope.root_node == node_id)
            .collect();
        scopes.sort_by_key(|scope| scope.declaration_order.unwrap_or(usize::MAX));
        scopes
    }

    fn ancestor_chain(&self, node_id: &NodeId) -> Vec<NodeId> {
        let mut reversed = Vec::new();
        let mut current = Some(node_id.clone());
        let mut seen = HashSet::new();

        while let Some(current_id) = current {
            if !seen.insert(current_id.clone()) {
                break;
            }
            reversed.push(current_id.clone());
            current = self.node(&current_id).and_then(|node| node.parent.clone());
        }

        reversed.reverse();
        reversed
    }

    pub fn is_descendant_or_self(&self, root: &NodeId, candidate: &NodeId) -> bool {
        let mut current = Some(candidate.clone());
        let mut seen = HashSet::new();

        while let Some(current_id) = current {
            if &current_id == root {
                return true;
            }
            if !seen.insert(current_id.clone()) {
                return false;
            }
            current = self.node(&current_id).and_then(|node| node.parent.clone());
        }

        false
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: NodeId,
    pub source_path: String,
    pub authored_source: AuthoredSource,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub properties: Vec<PropertyId>,
    pub explicit_scopes: Vec<ScopeId>,
    pub projections: Vec<ProjectionId>,
}

impl Node {
    pub fn new(id: impl Into<NodeId>, source_path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source_path: source_path.into(),
            authored_source: AuthoredSource::default(),
            parent: None,
            children: Vec::new(),
            properties: Vec::new(),
            explicit_scopes: Vec::new(),
            projections: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<NodeId>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = NodeId>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn with_authored_source(mut self, source: impl Into<String>) -> Self {
        self.authored_source = AuthoredSource::new(source);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthoredSource {
    pub typst: String,
}

impl AuthoredSource {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            typst: source.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub name: Option<String>,
    pub root_node: NodeId,
    pub extent: ScopeExtent,
    pub attributes: Vec<Attribute>,
    pub implicit: bool,
    pub declaration_order: Option<usize>,
}

impl Scope {
    pub fn global(root_node: NodeId) -> Self {
        Self {
            id: Publication::implicit_global_scope_id(),
            kind: ScopeKind::Global,
            name: Some("global".to_string()),
            root_node,
            extent: ScopeExtent::Descendants,
            attributes: Vec::new(),
            implicit: true,
            declaration_order: None,
        }
    }

    pub fn current_page(root_node: NodeId) -> Self {
        Self {
            id: Publication::implicit_current_page_scope_id(&root_node),
            kind: ScopeKind::CurrentPage,
            name: Some("current-page".to_string()),
            root_node,
            extent: ScopeExtent::CurrentNode,
            attributes: Vec::new(),
            implicit: true,
            declaration_order: None,
        }
    }

    pub fn explicit(
        id: impl Into<ScopeId>,
        kind: ScopeKind,
        root_node: impl Into<NodeId>,
        declaration_order: usize,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            name: None,
            root_node: root_node.into(),
            extent: ScopeExtent::Descendants,
            attributes: Vec::new(),
            implicit: false,
            declaration_order: Some(declaration_order),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_extent(mut self, extent: ScopeExtent) -> Self {
        self.extent = extent;
        self
    }

    pub fn covers(&self, publication: &Publication, node_id: &NodeId) -> bool {
        match &self.extent {
            ScopeExtent::Descendants => publication.is_descendant_or_self(&self.root_node, node_id),
            ScopeExtent::CurrentNode => &self.root_node == node_id,
            ScopeExtent::ExplicitNodes(nodes) => nodes.contains(node_id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    CurrentPage,
    Nav,
    Outline,
    Reference,
    Bibliography,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeExtent {
    Descendants,
    CurrentNode,
    ExplicitNodes(Vec<NodeId>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spine {
    pub node: NodeId,
    pub scopes: Vec<ScopeId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub id: PropertyId,
    pub key: String,
    pub value: Value,
    pub source: PropertySource,
    pub attributes: Vec<Attribute>,
    pub owning_node: NodeId,
}

impl Property {
    pub fn new(
        id: impl Into<PropertyId>,
        owning_node: impl Into<NodeId>,
        key: impl Into<String>,
        value: Value,
        source: PropertySource,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            value,
            source,
            attributes: Vec::new(),
            owning_node: owning_node.into(),
        }
    }

    pub fn is_implicit(&self) -> bool {
        self.source.is_implicit()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertySource {
    Implicit { reason: String },
    Heading { level: u8 },
    ExplicitPublisherCall { call: String },
    RawTypst { expression: String },
}

impl PropertySource {
    pub fn is_implicit(&self) -> bool {
        matches!(self, Self::Implicit { .. } | Self::Heading { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub id: QueryId,
    pub origin_node: NodeId,
    pub selection: SourcedValue<QuerySelection>,
    pub search: SourcedValue<QuerySearchRule>,
    pub filters: Vec<QueryFilter>,
    pub ordering: SourcedValue<QueryOrdering>,
    pub visibility: SourcedValue<QueryVisibility>,
    pub fallback: SourcedValue<QueryFallback>,
    pub ambiguity: SourcedValue<QueryAmbiguity>,
}

impl Query {
    pub fn new(id: impl Into<QueryId>, origin_node: impl Into<NodeId>) -> Self {
        Self {
            id: id.into(),
            origin_node: origin_node.into(),
            selection: SourcedValue::defaulted(QuerySelection::Nodes),
            search: SourcedValue::defaulted(QuerySearchRule::NearestScope),
            filters: Vec::new(),
            ordering: SourcedValue::defaulted(QueryOrdering::PublicationTree),
            visibility: SourcedValue::defaulted(QueryVisibility::VisibleFromOrigin),
            fallback: SourcedValue::defaulted(QueryFallback::Empty),
            ambiguity: SourcedValue::defaulted(QueryAmbiguity::AllowMany),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub id: ProjectionId,
    pub origin_node: NodeId,
    pub kind: ProjectionKind,
    pub query: QueryId,
    pub rendering_attributes: Vec<Attribute>,
    pub suppression: ProjectionSuppression,
}

impl Projection {
    pub fn new(
        id: impl Into<ProjectionId>,
        origin_node: impl Into<NodeId>,
        kind: ProjectionKind,
        query: impl Into<QueryId>,
    ) -> Self {
        Self {
            id: id.into(),
            origin_node: origin_node.into(),
            kind,
            query: query.into(),
            rendering_attributes: Vec::new(),
            suppression: ProjectionSuppression::NotSuppressed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcedValue<T> {
    pub value: T,
    pub source: ValueSource,
}

impl<T> SourcedValue<T> {
    pub fn explicit(value: T) -> Self {
        Self {
            value,
            source: ValueSource::Explicit,
        }
    }

    pub fn defaulted(value: T) -> Self {
        Self {
            value,
            source: ValueSource::Defaulted,
        }
    }

    pub fn is_explicit(&self) -> bool {
        self.source == ValueSource::Explicit
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueSource {
    Explicit,
    Defaulted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuerySelection {
    Nodes,
    Scopes,
    Properties,
    ResolvedTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuerySearchRule {
    CurrentNode,
    CurrentScope,
    NearestScope,
    ActiveScopes { order: ScopeSearchOrder },
    NamedScope(String),
    WholePublication,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeSearchOrder {
    NearestToGlobal,
    GlobalToNearest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryFilter {
    pub target: String,
    pub op: FilterOp,
    pub value: Value,
    pub source: ValueSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterOp {
    Equals,
    Contains,
    Raw(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryOrdering {
    PublicationTree,
    SourceOrder,
    Raw(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryVisibility {
    VisibleFromOrigin,
    IncludeHidden,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryFallback {
    Empty,
    Error,
    Raw(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryAmbiguity {
    AllowMany,
    Error,
    First,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    Outline,
    Navigation,
    Bibliography,
    Reference,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionSuppression {
    NotSuppressed,
    Suppressed { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub key: String,
    pub value: Value,
}

impl Attribute {
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    Bool(bool),
    Number(i64),
    TypstLabel(String),
    RawTypst(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseWarning {
    pub source_path: Option<String>,
    pub kind: ParseWarningKind,
    pub message: String,
}

impl ParseWarning {
    pub fn unreachable_typ_file(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            source_path: Some(path.clone()),
            kind: ParseWarningKind::UnreachableTypFile,
            message: format!("unreachable Typst source file: {path}"),
        }
    }

    pub fn preserved_raw_typst(
        source_path: impl Into<String>,
        expression: impl Into<String>,
    ) -> Self {
        let expression = expression.into();
        Self {
            source_path: Some(source_path.into()),
            kind: ParseWarningKind::PreservedRawTypstExpression,
            message: format!("preserved raw Typst expression: {expression}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseWarningKind {
    UnreachableTypFile,
    PreservedRawTypstExpression,
    UnsupportedPublisherCall,
}
