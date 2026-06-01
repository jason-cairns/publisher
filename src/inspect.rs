use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::model::*;
use crate::validation::{ValidationError, ValidationReport};

#[derive(Clone, Debug, Default)]
pub struct InspectOptions {
    pub debug_defaults: bool,
    pub source_root: Option<PathBuf>,
}

impl InspectOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn debug_defaults(mut self, debug_defaults: bool) -> Self {
        self.debug_defaults = debug_defaults;
        self
    }

    pub fn source_root(mut self, source_root: impl Into<PathBuf>) -> Self {
        self.source_root = Some(source_root.into());
        self
    }
}

pub fn inspect_publication(
    publication: &Publication,
    report: &ValidationReport,
    options: &InspectOptions,
) -> String {
    let mut out = String::new();

    write_summary(&mut out, publication, report);
    write_tree(&mut out, publication);
    write_node_details(&mut out, publication, options);
    write_scopes(&mut out, publication);
    write_spines(&mut out, publication);
    write_properties(&mut out, publication);
    write_parse_warnings(&mut out, report);
    write_validation_result(&mut out, report);

    out
}

fn write_summary(out: &mut String, publication: &Publication, report: &ValidationReport) {
    writeln!(out, "Publication").unwrap();
    writeln!(
        out,
        "  root: {}",
        source_path(publication, &publication.root_node)
    )
    .unwrap();
    writeln!(out, "  nodes: {}", publication.nodes.len()).unwrap();
    writeln!(out, "  scopes: {}", publication.scopes.len()).unwrap();
    writeln!(out, "  properties: {}", publication.properties.len()).unwrap();
    writeln!(out, "  warnings: {}", report.warnings.len()).unwrap();
    writeln!(out, "  validation-errors: {}", report.errors.len()).unwrap();
}

fn write_tree(out: &mut String, publication: &Publication) {
    writeln!(out).unwrap();
    writeln!(out, "Tree").unwrap();
    write_tree_node(out, publication, &publication.root_node, 1);
}

fn write_tree_node(out: &mut String, publication: &Publication, node_id: &NodeId, depth: usize) {
    let indent = "  ".repeat(depth);
    writeln!(out, "{indent}{}", source_path(publication, node_id)).unwrap();
    if let Some(node) = publication.node(node_id) {
        for child in &node.children {
            write_tree_node(out, publication, child, depth + 1);
        }
    }
}

fn write_node_details(out: &mut String, publication: &Publication, options: &InspectOptions) {
    for node in &publication.nodes {
        writeln!(out).unwrap();
        writeln!(out, "Node {}", node.source_path).unwrap();
        writeln!(
            out,
            "  id: {}{}",
            node.id,
            match &node.parent {
                Some(parent) => format!(" parent={}", source_path(publication, parent)),
                None => " parent=<root>".to_string(),
            }
        )
        .unwrap();
        writeln!(
            out,
            "  parsed-from: {}",
            parsed_from(&node.source_path, options.source_root.as_deref())
        )
        .unwrap();
        writeln!(out, "  html-route: {}", node.html_route).unwrap();
        write_node_children(out, publication, node);
        write_node_properties(out, publication, node);
        write_node_scopes(out, publication, node);
        write_node_spine(out, publication, &node.id);
    }
}

fn write_node_children(out: &mut String, publication: &Publication, node: &Node) {
    writeln!(out, "  children:").unwrap();
    if node.children.is_empty() {
        writeln!(out, "    <none>").unwrap();
    } else {
        for child in &node.children {
            writeln!(out, "    {}", source_path(publication, child)).unwrap();
        }
    }
}

fn write_node_properties(out: &mut String, publication: &Publication, node: &Node) {
    writeln!(out, "  properties:").unwrap();
    if node.properties.is_empty() {
        writeln!(out, "    <none>").unwrap();
    } else {
        for property_id in &node.properties {
            match publication.property(property_id) {
                Some(property) => writeln!(out, "    {}", format_property(property)).unwrap(),
                None => writeln!(out, "    <missing property {property_id}>").unwrap(),
            }
        }
    }
}

fn write_node_scopes(out: &mut String, publication: &Publication, node: &Node) {
    writeln!(out, "  scopes declared:").unwrap();
    if node.explicit_scopes.is_empty() {
        writeln!(out, "    <none>").unwrap();
    } else {
        for scope_id in &node.explicit_scopes {
            match publication.scope(scope_id) {
                Some(scope) => writeln!(out, "    {}", format_scope(scope, publication)).unwrap(),
                None => writeln!(out, "    <missing scope {scope_id}>").unwrap(),
            }
        }
    }
}

fn write_node_spine(out: &mut String, publication: &Publication, node_id: &NodeId) {
    writeln!(out, "  spine:").unwrap();
    match publication.spine_for(node_id) {
        Ok(spine) => {
            for scope_id in spine.scopes {
                writeln!(out, "    {}", format_scope_ref(publication, &scope_id)).unwrap();
            }
        }
        Err(error) => writeln!(out, "    <error: {}>", format_validation_error(&error)).unwrap(),
    }
}

fn write_scopes(out: &mut String, publication: &Publication) {
    writeln!(out).unwrap();
    writeln!(out, "Scopes").unwrap();
    for scope in &publication.scopes {
        writeln!(out, "  {}", format_scope(scope, publication)).unwrap();
    }
}

fn write_spines(out: &mut String, publication: &Publication) {
    writeln!(out).unwrap();
    writeln!(out, "Derived Spines").unwrap();
    for node in &publication.nodes {
        match publication.spine_for(&node.id) {
            Ok(spine) => {
                let scopes = spine
                    .scopes
                    .iter()
                    .map(|scope_id| format_scope_ref(publication, scope_id))
                    .collect::<Vec<_>>()
                    .join(" -> ");
                writeln!(out, "  {}: {scopes}", node.source_path).unwrap();
            }
            Err(error) => {
                writeln!(
                    out,
                    "  {}: <error: {}>",
                    node.source_path,
                    format_validation_error(&error)
                )
                .unwrap();
            }
        }
    }
}

fn write_properties(out: &mut String, publication: &Publication) {
    writeln!(out).unwrap();
    writeln!(out, "Properties").unwrap();
    for property in &publication.properties {
        writeln!(
            out,
            "  {} owns {}",
            source_path(publication, &property.owning_node),
            format_property(property)
        )
        .unwrap();
    }
}

fn write_parse_warnings(out: &mut String, report: &ValidationReport) {
    writeln!(out).unwrap();
    writeln!(out, "Parse Warnings").unwrap();
    if report.warnings.is_empty() {
        writeln!(out, "  <none>").unwrap();
    } else {
        for warning in &report.warnings {
            writeln!(
                out,
                "  {}{}: {}",
                parse_warning_kind(&warning.kind),
                warning
                    .source_path
                    .as_ref()
                    .map(|path| format!(" {path}"))
                    .unwrap_or_default(),
                warning.message
            )
            .unwrap();
        }
    }
}

fn write_validation_result(out: &mut String, report: &ValidationReport) {
    writeln!(out).unwrap();
    writeln!(out, "Validation Result").unwrap();
    if report.errors.is_empty() {
        writeln!(out, "  ok").unwrap();
    } else {
        writeln!(out, "  errors: {}", report.errors.len()).unwrap();
        for error in &report.errors {
            writeln!(out, "  - {}", format_validation_error(error)).unwrap();
        }
    }
}

fn parsed_from(source_path: &str, source_root: Option<&Path>) -> String {
    source_root
        .map(|root| root.join(source_path).display().to_string())
        .unwrap_or_else(|| source_path.to_string())
}

fn source_path(publication: &Publication, node_id: &NodeId) -> String {
    publication
        .node(node_id)
        .map(|node| node.source_path.clone())
        .unwrap_or_else(|| node_id.to_string())
}

fn format_scope(scope: &Scope, publication: &Publication) -> String {
    format!(
        "{} kind={} name={} root={} extent={} {} order={}",
        scope.id,
        scope_kind(&scope.kind),
        scope.name.as_deref().unwrap_or("<none>"),
        source_path(publication, &scope.root_node),
        scope_extent(&scope.extent),
        if scope.implicit {
            "implicit"
        } else {
            "explicit"
        },
        scope
            .declaration_order
            .map(|order| order.to_string())
            .unwrap_or_else(|| "<none>".to_string())
    )
}

fn format_scope_ref(publication: &Publication, scope_id: &ScopeId) -> String {
    match publication.scope(scope_id) {
        Some(scope) => format!(
            "{}({})",
            scope_kind(&scope.kind),
            source_path(publication, &scope.root_node)
        ),
        None => format!("<missing {scope_id}>"),
    }
}

fn format_property(property: &Property) -> String {
    let attributes = if property.attributes.is_empty() {
        String::new()
    } else {
        format!(" attrs=[{}]", format_attributes(&property.attributes))
    };
    format!(
        "{} = {} {}{}",
        property.key,
        format_value(&property.value),
        property_source(&property.source),
        attributes
    )
}

fn format_attributes(attributes: &[Attribute]) -> String {
    attributes
        .iter()
        .map(|attribute| format!("{}={}", attribute.key, format_value(&attribute.value)))
        .collect::<Vec<_>>()
        .join(", ")
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

fn property_source(source: &PropertySource) -> String {
    match source {
        PropertySource::Implicit { reason } => format!("implicit({reason})"),
        PropertySource::Heading { level } => format!("implicit(source-heading level={level})"),
        PropertySource::ExplicitPublisherCall { call } => format!("explicit({call})"),
        PropertySource::RawTypst { expression } => format!("explicit(raw {expression})"),
    }
}

fn scope_kind(kind: &ScopeKind) -> String {
    match kind {
        ScopeKind::Global => "global".to_string(),
        ScopeKind::CurrentPage => "current-page".to_string(),
        ScopeKind::Publication => "publication".to_string(),
    }
}

fn scope_extent(extent: &ScopeExtent) -> String {
    match extent {
        ScopeExtent::Descendants => "descendants".to_string(),
        ScopeExtent::CurrentNode => "current-node".to_string(),
        ScopeExtent::ExplicitNodes(nodes) => format!(
            "explicit-nodes({})",
            nodes
                .iter()
                .map(NodeId::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn parse_warning_kind(kind: &ParseWarningKind) -> &'static str {
    match kind {
        ParseWarningKind::UnreachableTypFile => "unreachable-typ-file",
        ParseWarningKind::PreservedRawTypstExpression => "preserved-raw-typst-expression",
        ParseWarningKind::UnsupportedPublisherCall => "unsupported-publisher-call",
        ParseWarningKind::DuplicateScopeTitle => "duplicate-scope-title",
        ParseWarningKind::DuplicateBibliography => "duplicate-bibliography",
    }
}

fn format_validation_error(error: &ValidationError) -> String {
    match error {
        ValidationError::DuplicateNodeSourcePath {
            source_path,
            first,
            second,
        } => format!("duplicate node source path {source_path}: {first} and {second}"),
        ValidationError::MissingChildTarget { parent, child } => {
            format!("missing child target {child} declared by {parent}")
        }
        ValidationError::MultipleParents {
            child,
            first_parent,
            second_parent,
        } => format!("multiple parents for {child}: {first_parent} and {second_parent}"),
        ValidationError::DuplicateScopeId { scope_id } => format!("duplicate scope id {scope_id}"),
        ValidationError::InvalidScopeRoot {
            scope_id,
            root_node,
        } => format!("invalid root node {root_node} for scope {scope_id}"),
        ValidationError::MissingPropertyOwner { property_id, owner } => {
            format!("property {property_id} has missing owner {owner}")
        }
        ValidationError::MissingNode { node_id } => format!("missing node {node_id}"),
        ValidationError::MissingScope { scope_id } => format!("missing scope {scope_id}"),
    }
}
