use publisher::*;

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
        ScopeKind::Publication,
        "index",
        0,
    ));
    publication.add_scope(Scope::explicit(
        "duplicate-scope",
        ScopeKind::Publication,
        "index",
        1,
    ));
    publication.add_scope(Scope::explicit(
        "invalid-root",
        ScopeKind::Publication,
        "missing-root",
        2,
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
    assert_eq!(
        report.warnings,
        vec![ParseWarning::unreachable_typ_file("unused.typ")]
    );
}

#[test]
fn scopes_overlap_and_warn_on_duplicate_display_titles() {
    let publication = parse_publication("examples/discovery_site/index.typ").unwrap();
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

#[test]
fn inspect_snapshot_is_stable() {
    let publication = parse_publication("examples/discovery_site/index.typ").unwrap();
    let report = publication.validate();
    let options = InspectOptions::new().source_root("examples/discovery_site");

    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    insta::assert_snapshot!(inspect_publication(&publication, &report, &options));
}
