use std::fmt::Write;

use crate::model::*;

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
