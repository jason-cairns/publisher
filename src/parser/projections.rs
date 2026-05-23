use std::collections::BTreeMap;

use super::calls::CallArgs;
use super::{ParsedFile, scope_kind, unique_scope_id};
use crate::model::*;

pub(super) fn record_scope(parsed: &mut ParsedFile, args: &CallArgs) {
    let Some(kind) = args.named_string("kind") else {
        return;
    };

    let scope_kind = scope_kind(&kind);
    let scope_id = unique_scope_id(&kind, &parsed.node_id, &parsed.scopes);
    let scope_name = args.named_string("name").unwrap_or_else(|| kind.clone());
    let scope = Scope::explicit(
        scope_id,
        scope_kind,
        parsed.node_id.clone(),
        parsed.declaration_order,
    )
    .with_name(scope_name);
    parsed.declaration_order += 1;
    parsed.scopes.push(scope);
}

pub(super) fn record_outline(parsed: &mut ParsedFile, args: &CallArgs) {
    let query_id = parsed.next_query_id("outline");
    let projection_id = parsed.next_projection_id("outline");

    let mut query = Query::new(query_id.clone(), parsed.node_id.clone());
    query.selection = SourcedValue::explicit(QuerySelection::Nodes);
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
        parsed.node_id.clone(),
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

    parsed.queries.push(query);
    parsed.projections.push(projection);
}

pub(super) fn record_bibliography(parsed: &mut ParsedFile, args: &CallArgs) {
    let query_id = parsed.next_query_id("bibliography");
    let projection_id = parsed.next_projection_id("bibliography");

    let mut query = Query::new(query_id.clone(), parsed.node_id.clone());
    query.selection = SourcedValue::explicit(QuerySelection::Properties);
    if args.named_string("scope").as_deref() == Some("current-page") {
        query.search = SourcedValue::explicit(QuerySearchRule::CurrentNode);
    }
    query.filters.push(QueryFilter {
        target: "property.key".to_string(),
        op: FilterOp::Equals,
        value: Value::String("citation".to_string()),
        source: ValueSource::Explicit,
    });

    let mut projection = Projection::new(
        projection_id,
        parsed.node_id.clone(),
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

    parsed.queries.push(query);
    parsed.projections.push(projection);
}

pub(super) fn record_reference(parsed: &mut ParsedFile, args: &CallArgs) {
    let Some(target) = args.first_label() else {
        return;
    };

    let query_id = parsed.next_query_id("reference");
    let projection_id = parsed.next_projection_id("reference");
    let mut query = Query::new(query_id.clone(), parsed.node_id.clone());
    query.selection = SourcedValue::explicit(QuerySelection::ResolvedTarget);
    query.filters.push(QueryFilter {
        target: "target".to_string(),
        op: FilterOp::Equals,
        value: Value::TypstLabel(target.clone()),
        source: ValueSource::Explicit,
    });

    let mut projection = Projection::new(
        projection_id,
        parsed.node_id.clone(),
        ProjectionKind::Reference,
        query_id,
    );
    projection
        .rendering_attributes
        .push(Attribute::new("target", Value::TypstLabel(target)));

    parsed.queries.push(query);
    parsed.projections.push(projection);
}

pub(super) fn record_nav_suppression(parsed: &mut ParsedFile) {
    parsed.nav_suppressed = true;
}

pub(super) fn add_inherited_nav_projections(
    publication: &mut Publication,
    parsed_files: &BTreeMap<String, ParsedFile>,
    reachable_order: &[String],
) {
    for source_path in reachable_order {
        let Some(parsed) = parsed_files.get(source_path) else {
            continue;
        };
        let Ok(spine) = publication.spine_for(&parsed.node_id) else {
            continue;
        };

        let nav_scope_ids = spine
            .scopes
            .iter()
            .filter(|scope_id| {
                publication
                    .scope(scope_id)
                    .is_some_and(|scope| scope.kind == ScopeKind::Nav)
            })
            .cloned()
            .collect::<Vec<_>>();

        for (index, scope_id) in nav_scope_ids.iter().enumerate() {
            let Some(scope) = publication.scope(scope_id) else {
                continue;
            };
            let mut query = Query::new(
                inherited_nav_query_id(&parsed.node_id, scope_id, index),
                parsed.node_id.clone(),
            );
            query.selection = SourcedValue::explicit(QuerySelection::Nodes);
            query.search =
                SourcedValue::explicit(QuerySearchRule::NamedScope(scope_id.as_str().to_string()));
            query.filters.push(QueryFilter {
                target: "scope.id".to_string(),
                op: FilterOp::Equals,
                value: Value::String(scope_id.as_str().to_string()),
                source: ValueSource::Explicit,
            });

            let mut projection = Projection::new(
                inherited_nav_projection_id(&parsed.node_id, scope_id, index),
                parsed.node_id.clone(),
                ProjectionKind::Navigation,
                query.id.clone(),
            );
            projection.rendering_attributes.push(Attribute::new(
                "scope",
                Value::String(scope_id.as_str().to_string()),
            ));
            projection.rendering_attributes.push(Attribute::new(
                "scope-root",
                Value::String(scope.root_node.as_str().to_string()),
            ));
            if parsed.nav_suppressed {
                projection.suppression = ProjectionSuppression::Suppressed {
                    reason: "publisher.nav.suppress()".to_string(),
                };
            }

            publication.add_query(query);
            publication.add_projection(projection);
        }
    }
}

fn inherited_nav_query_id(node_id: &NodeId, scope_id: &ScopeId, index: usize) -> QueryId {
    QueryId::new(format!("query:{node_id}:nav:{scope_id}:{}", index + 1))
}

fn inherited_nav_projection_id(node_id: &NodeId, scope_id: &ScopeId, index: usize) -> ProjectionId {
    ProjectionId::new(format!("projection:{node_id}:nav:{scope_id}:{}", index + 1))
}
