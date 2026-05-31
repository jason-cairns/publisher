use publisher::*;

#[test]
fn current_page_scopes_do_not_flow_to_children() {
    let publication = representative_publication();

    let root_spine = publication.spine_for(&NodeId::from("index")).unwrap();
    let child_spine = publication.spine_for(&NodeId::from("writing")).unwrap();

    assert!(
        root_spine
            .scopes
            .contains(&ScopeId::from("current-page:index"))
    );
    assert!(
        !child_spine
            .scopes
            .contains(&ScopeId::from("current-page:index"))
    );
    assert!(
        child_spine
            .scopes
            .contains(&ScopeId::from("current-page:writing"))
    );
}

#[test]
fn spines_are_ordered_global_to_nearest() {
    let publication = representative_publication();

    let spine = publication.spine_for(&NodeId::from("writing")).unwrap();

    assert_eq!(
        spine.scopes,
        vec![
            ScopeId::from("global"),
            ScopeId::from("nav:index"),
            ScopeId::from("current-page:writing"),
            ScopeId::from("outline:writing"),
            ScopeId::from("reference:writing"),
        ]
    );
}

#[test]
fn nav_suppression_is_projection_owned() {
    let publication = representative_publication();

    assert!(
        !publication
            .properties
            .iter()
            .any(|property| property.key == "nav.suppressed")
    );

    let global_scope = publication.scope(&ScopeId::from("global")).unwrap();
    assert!(global_scope.attributes.is_empty());
    assert!(!publication.scopes.iter().any(|scope| {
        scope
            .attributes
            .iter()
            .any(|attribute| attribute.key == "nav.suppressed")
    }));

    let context = publication
        .property_scope_context(&PropertyId::from("prop:index:source-path"))
        .unwrap();
    assert_eq!(
        context.scopes,
        vec![
            ScopeId::from("global"),
            ScopeId::from("current-page:index"),
            ScopeId::from("nav:index")
        ]
    );

    let nav_projection = publication
        .projection(&ProjectionId::from("projection:index:nav"))
        .unwrap();
    assert_eq!(
        nav_projection.suppression,
        ProjectionSuppression::Suppressed {
            reason: "publisher.nav.suppress()".to_string()
        }
    );
}

#[test]
fn validation_reports_core_model_errors_and_parse_warnings() {
    let mut publication = Publication::new(Node::new("index", "index.typ"));
    publication.add_node(Node::new("dup", "index.typ"));

    let mut parent_a = Node::new("a", "a.typ");
    parent_a.children = vec![NodeId::from("missing"), NodeId::from("child")];
    publication.add_node(parent_a);

    let mut parent_b = Node::new("b", "b.typ");
    parent_b.children = vec![NodeId::from("child")];
    publication.add_node(parent_b);
    publication.add_node(Node::new("child", "child.typ"));

    publication.add_scope(Scope::explicit(
        "duplicate-scope",
        ScopeKind::Nav,
        "index",
        0,
    ));
    publication.add_scope(Scope::explicit(
        "duplicate-scope",
        ScopeKind::Outline,
        "index",
        1,
    ));
    publication.add_scope(Scope::explicit(
        "invalid-root",
        ScopeKind::Reference,
        "missing-root",
        2,
    ));

    publication.add_query(Query::new("query:missing-origin", "missing-origin"));
    publication.add_projection(Projection::new(
        "projection:missing-origin",
        "missing-origin",
        ProjectionKind::Outline,
        "query:missing",
    ));
    publication.add_parse_warning(ParseWarning::unreachable_typ_file("unused.typ"));

    let report = publication.validate();

    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::DuplicateNodeSourcePath { source_path, .. } if source_path == "index.typ"
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::MissingChildTarget { child, .. } if child == &NodeId::from("missing")
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::MultipleParents { child, .. } if child == &NodeId::from("child")
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::DuplicateScopeId { scope_id } if scope_id == &ScopeId::from("duplicate-scope")
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::InvalidScopeRoot { scope_id, .. } if scope_id == &ScopeId::from("invalid-root")
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::MissingQueryOrigin { query_id, .. } if query_id == &QueryId::from("query:missing-origin")
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::MissingProjectionOrigin { projection_id, .. } if projection_id == &ProjectionId::from("projection:missing-origin")
    )));
    assert!(report.errors.iter().any(|error| matches!(
        error,
        ValidationError::MissingProjectionQuery { projection_id, .. } if projection_id == &ProjectionId::from("projection:missing-origin")
    )));
    assert_eq!(
        report.warnings,
        vec![ParseWarning::unreachable_typ_file("unused.typ")]
    );
}

#[test]
fn prd003_scopes_overlap_and_warn_on_duplicate_display_titles() {
    let publication = parse_publication("examples/prd003_discovery_site/index.typ").unwrap();
    let report = publication.validate();

    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:#?}",
        report.errors
    );

    assert_eq!(
        publication
            .spine_for(&NodeId::from("writing/blog-2"))
            .unwrap()
            .scopes,
        vec![
            ScopeId::from("global"),
            ScopeId::from("home"),
            ScopeId::from("writing"),
            Publication::implicit_source_document_scope_id(&NodeId::from("writing/blog-2")),
            ScopeId::from("duplicate-title-a"),
        ]
    );
    assert_eq!(
        publication
            .spine_for(&NodeId::from("thesis/ch-1"))
            .unwrap()
            .scopes,
        vec![
            ScopeId::from("global"),
            ScopeId::from("home"),
            ScopeId::from("thesis"),
            Publication::implicit_source_document_scope_id(&NodeId::from("thesis/ch-1")),
            ScopeId::from("duplicate-title-b"),
        ]
    );
    assert!(report.warnings.iter().any(|warning| {
        warning.kind == ParseWarningKind::DuplicateScopeTitle
            && warning.message
                == "duplicate scope title \"Writing\" used by scope ids: writing, duplicate-title-a, duplicate-title-b"
    }));
    assert!(report.warnings.iter().any(|warning| {
        warning.kind == ParseWarningKind::DuplicateBibliography
            && warning.message
                == "duplicate bibliography calls in scope writing: writing/blog-1.typ, writing/blog-2.typ"
    }));
    assert!(report.warnings.iter().any(|warning| {
        warning.kind == ParseWarningKind::DuplicateBibliography
            && warning.message
                == "duplicate bibliography calls in scope thesis: thesis/intro.typ, thesis/ch-1.typ"
    }));
}

fn representative_publication() -> Publication {
    let mut root = Node::new("index", "index.typ");
    root.children = vec![NodeId::from("writing")];

    let mut publication = Publication::new(root);
    publication.add_node(Node::new("writing", "writing.typ").with_parent("index"));

    publication.add_scope(Scope::explicit("nav:index", ScopeKind::Nav, "index", 0));
    publication.add_scope(Scope::explicit(
        "outline:writing",
        ScopeKind::Outline,
        "writing",
        0,
    ));
    publication.add_scope(Scope::explicit(
        "reference:writing",
        ScopeKind::Reference,
        "writing",
        1,
    ));

    publication.add_property(Property::new(
        "prop:index:source-path",
        "index",
        "source-path",
        Value::String("index.typ".to_string()),
        PropertySource::Implicit {
            reason: "source path".to_string(),
        },
    ));
    publication.add_query(Query::new("query:index:nav", "index"));
    publication.add_projection(Projection {
        id: ProjectionId::from("projection:index:nav"),
        origin_node: NodeId::from("index"),
        kind: ProjectionKind::Navigation,
        query: QueryId::from("query:index:nav"),
        rendering_attributes: Vec::new(),
        suppression: ProjectionSuppression::Suppressed {
            reason: "publisher.nav.suppress()".to_string(),
        },
    });

    publication
}
