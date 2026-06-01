use super::ParsedFile;
use crate::model::*;

pub(super) fn record_publication_scope(
    parsed: &mut ParsedFile,
    id: String,
    title: Option<String>,
    tags: Vec<String>,
) {
    let mut scope = Scope::explicit(
        id,
        ScopeKind::Publication,
        parsed.node_id.clone(),
        parsed.declaration_order,
    );
    if let Some(title) = title {
        scope = scope.with_name(title);
    }
    scope.attributes.extend(
        tags.into_iter()
            .map(|tag| Attribute::new("tag", Value::String(tag))),
    );
    parsed.declaration_order += 1;
    parsed.scopes.push(scope);
}

pub(super) fn record_scope_payload(
    parsed: &mut ParsedFile,
    kind: String,
    value: String,
    depth: Option<i64>,
) {
    let Some(scope_id) = nearest_explicit_publication_scope(parsed) else {
        parsed.warnings.push(ParseWarning {
            source_path: Some(parsed.source_path.clone()),
            kind: ParseWarningKind::UnsupportedPublisherCall,
            message: format!(
                "scope payload {kind:?} has no preceding publication scope in {}",
                parsed.source_path
            ),
        });
        return;
    };

    let mut payload = ScopePayload::new(
        scope_id,
        parsed.source_path.clone(),
        kind,
        Value::String(value),
        parsed.declaration_order,
    );
    if let Some(depth) = depth {
        payload = payload.with_attribute("depth", Value::Number(depth));
    }
    parsed.payloads.push(payload);
    parsed.declaration_order += 1;
}

fn nearest_explicit_publication_scope(parsed: &ParsedFile) -> Option<ScopeId> {
    parsed
        .scopes
        .iter()
        .rev()
        .find(|scope| !scope.implicit && scope.kind == ScopeKind::Publication)
        .map(|scope| scope.id.clone())
}
