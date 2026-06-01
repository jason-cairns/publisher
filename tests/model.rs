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
fn publish_glob_expands_children_in_sorted_order() {
    let publication = parse_publication("examples/discovery_site/index.typ").unwrap();
    let writing = publication.node(&NodeId::from("writing")).unwrap();

    assert_eq!(
        writing.children,
        vec![
            NodeId::from("writing/blog-1"),
            NodeId::from("writing/blog-2")
        ]
    );
}

#[test]
fn scope_payloads_are_returned_in_spine_order() {
    let mut publication = Publication::new(Node::new("index", "index.typ"));
    let mut writing = Node::new("writing", "writing.typ").with_parent("index");
    writing.children = vec![NodeId::from("writing/post")];
    let post = Node::new("writing/post", "writing/post.typ").with_parent("writing");
    publication
        .node_mut(&NodeId::from("index"))
        .unwrap()
        .children = vec![NodeId::from("writing")];
    publication.add_node(writing);
    publication.add_node(post);

    publication.add_scope(Scope::explicit("home", ScopeKind::Publication, "index", 0));
    publication.add_scope(Scope::explicit(
        "writing",
        ScopeKind::Publication,
        "writing",
        1,
    ));
    publication.add_scope_payload(ScopePayload::new(
        "writing",
        "writing.typ",
        "nav",
        Value::String("section".to_string()),
        1,
    ));
    publication.add_scope_payload(ScopePayload::new(
        "home",
        "index.typ",
        "nav",
        Value::String("site".to_string()),
        0,
    ));
    publication.add_scope_payload(ScopePayload::new(
        "home",
        "index.typ",
        "css",
        Value::String("site.css".to_string()),
        2,
    ));

    let payloads = publication
        .payloads_for(&NodeId::from("writing/post"), "nav")
        .unwrap();

    assert_eq!(
        payloads
            .iter()
            .map(|payload| match &payload.value {
                Value::String(value) => value.as_str(),
                other => panic!("unexpected payload value: {other:?}"),
            })
            .collect::<Vec<_>>(),
        vec!["site", "section"]
    );
}

#[test]
fn decoded_scope_payload_attaches_to_nearest_preceding_scope() {
    let temp = std::env::temp_dir().join(format!(
        "publisher-payload-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    std::fs::write(
        temp.join("index.typ"),
        r#"#metadata((kind: "scope", id: "home", title: none, tags: ())) <publisher-marker>
#metadata((kind: "payload", payload_kind: "nav", value: "site")) <publisher-marker>
#metadata((kind: "scope", id: "writing", title: none, tags: ())) <publisher-marker>
#metadata((kind: "payload", payload_kind: "nav", value: "section")) <publisher-marker>
"#,
    )
    .unwrap();

    let publication = parse_publication(temp.join("index.typ")).unwrap();
    let report = publication.validate();
    assert!(
        report.errors.is_empty(),
        "expected valid payload fixture, got {:#?}",
        report.errors
    );

    let payloads = publication
        .payloads_for(&NodeId::from("index"), "nav")
        .unwrap();

    assert_eq!(
        payloads
            .iter()
            .map(|payload| {
                let Value::String(value) = &payload.value else {
                    panic!("unexpected payload value: {:?}", payload.value);
                };
                (payload.scope_id.as_str(), value.as_str())
            })
            .collect::<Vec<_>>(),
        vec![("home", "site"), ("writing", "section")]
    );
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
