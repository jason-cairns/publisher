use std::collections::{BTreeMap, BTreeSet};

use crate::model::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ParseWarning>,
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateNodeSourcePath {
        source_path: String,
        first: NodeId,
        second: NodeId,
    },
    MissingChildTarget {
        parent: NodeId,
        child: NodeId,
    },
    MultipleParents {
        child: NodeId,
        first_parent: NodeId,
        second_parent: NodeId,
    },
    DuplicateScopeId {
        scope_id: ScopeId,
    },
    InvalidScopeRoot {
        scope_id: ScopeId,
        root_node: NodeId,
    },
    MissingProjectionOrigin {
        projection_id: ProjectionId,
        origin: NodeId,
    },
    MissingProjectionQuery {
        projection_id: ProjectionId,
        query_id: QueryId,
    },
    MissingQueryOrigin {
        query_id: QueryId,
        origin: NodeId,
    },
    MissingPropertyOwner {
        property_id: PropertyId,
        owner: NodeId,
    },
    MissingNode {
        node_id: NodeId,
    },
    MissingScope {
        scope_id: ScopeId,
    },
}

pub fn validate_publication(publication: &Publication) -> ValidationReport {
    let mut report = ValidationReport {
        errors: Vec::new(),
        warnings: publication.parse_warnings.clone(),
    };

    validate_duplicate_source_paths(publication, &mut report);
    validate_children(publication, &mut report);
    validate_scopes(publication, &mut report);
    validate_duplicate_publication_scope_titles(publication, &mut report);
    validate_properties(publication, &mut report);
    validate_queries(publication, &mut report);
    validate_projections(publication, &mut report);

    report
}

fn validate_duplicate_source_paths(publication: &Publication, report: &mut ValidationReport) {
    let mut seen: BTreeMap<&str, &NodeId> = BTreeMap::new();
    for node in &publication.nodes {
        if let Some(first) = seen.insert(node.source_path.as_str(), &node.id) {
            report
                .errors
                .push(ValidationError::DuplicateNodeSourcePath {
                    source_path: node.source_path.clone(),
                    first: first.clone(),
                    second: node.id.clone(),
                });
        }
    }
}

fn validate_children(publication: &Publication, report: &mut ValidationReport) {
    let node_ids: BTreeSet<_> = publication
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect();
    let mut parents_by_child: BTreeMap<NodeId, NodeId> = BTreeMap::new();

    for parent in &publication.nodes {
        for child_id in &parent.children {
            if !node_ids.contains(child_id) {
                report.errors.push(ValidationError::MissingChildTarget {
                    parent: parent.id.clone(),
                    child: child_id.clone(),
                });
                continue;
            }

            if let Some(first_parent) = parents_by_child.insert(child_id.clone(), parent.id.clone())
            {
                if first_parent != parent.id {
                    report.errors.push(ValidationError::MultipleParents {
                        child: child_id.clone(),
                        first_parent,
                        second_parent: parent.id.clone(),
                    });
                }
            }
        }
    }
}

fn validate_scopes(publication: &Publication, report: &mut ValidationReport) {
    let node_ids: BTreeSet<_> = publication
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect();
    let mut scope_ids = BTreeSet::new();

    for scope in &publication.scopes {
        if !scope_ids.insert(scope.id.clone()) {
            report.errors.push(ValidationError::DuplicateScopeId {
                scope_id: scope.id.clone(),
            });
        }

        if !node_ids.contains(&scope.root_node) {
            report.errors.push(ValidationError::InvalidScopeRoot {
                scope_id: scope.id.clone(),
                root_node: scope.root_node.clone(),
            });
        }

        if let ScopeExtent::ExplicitNodes(nodes) = &scope.extent {
            for node_id in nodes {
                if !node_ids.contains(node_id) {
                    report.errors.push(ValidationError::MissingNode {
                        node_id: node_id.clone(),
                    });
                }
            }
        }
    }
}

fn validate_duplicate_publication_scope_titles(
    publication: &Publication,
    report: &mut ValidationReport,
) {
    let mut titled_scopes: BTreeMap<&str, Vec<ScopeId>> = BTreeMap::new();

    for scope in &publication.scopes {
        if scope.implicit || scope.kind != ScopeKind::Publication {
            continue;
        }
        let Some(title) = scope.name.as_deref() else {
            continue;
        };
        titled_scopes
            .entry(title)
            .or_default()
            .push(scope.id.clone());
    }

    for (title, scope_ids) in titled_scopes {
        if scope_ids.len() > 1 {
            report
                .warnings
                .push(ParseWarning::duplicate_scope_title(title, scope_ids));
        }
    }
}

fn validate_properties(publication: &Publication, report: &mut ValidationReport) {
    for property in &publication.properties {
        if publication.node(&property.owning_node).is_none() {
            report.errors.push(ValidationError::MissingPropertyOwner {
                property_id: property.id.clone(),
                owner: property.owning_node.clone(),
            });
        }
    }
}

fn validate_queries(publication: &Publication, report: &mut ValidationReport) {
    for query in &publication.queries {
        if publication.node(&query.origin_node).is_none() {
            report.errors.push(ValidationError::MissingQueryOrigin {
                query_id: query.id.clone(),
                origin: query.origin_node.clone(),
            });
        }
    }
}

fn validate_projections(publication: &Publication, report: &mut ValidationReport) {
    for projection in &publication.projections {
        if publication.node(&projection.origin_node).is_none() {
            report
                .errors
                .push(ValidationError::MissingProjectionOrigin {
                    projection_id: projection.id.clone(),
                    origin: projection.origin_node.clone(),
                });
        }

        if publication.query(&projection.query).is_none() {
            report.errors.push(ValidationError::MissingProjectionQuery {
                projection_id: projection.id.clone(),
                query_id: projection.query.clone(),
            });
        }
    }
}
