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
