use publisher::*;

#[test]
fn parse_api_sketch_fixture_builds_expected_publication() {
    let publication = parse_publication("examples/api_sketch_site/index.typ").unwrap();
    let report = publication.validate();

    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    let node_paths: Vec<_> = publication
        .nodes
        .iter()
        .map(|node| node.source_path.as_str())
        .collect();
    assert_eq!(
        node_paths,
        vec![
            "index.typ",
            "writing.typ",
            "writing/blog-1.typ",
            "writing/blog-2.typ",
            "thesis/intro.typ",
            "thesis/bib.typ",
            "thesis/ch-1.typ",
            "cv.typ",
        ]
    );

    assert_children(&publication, "index", &["writing", "thesis/intro", "cv"]);
    assert_children(
        &publication,
        "writing",
        &["writing/blog-1", "writing/blog-2"],
    );
    assert_children(&publication, "thesis/intro", &["thesis/bib", "thesis/ch-1"]);

    assert_scope(&publication, "nav:index", ScopeKind::Nav, "index");
    assert_scope(
        &publication,
        "outline:writing",
        ScopeKind::Outline,
        "writing",
    );
    assert_scope(
        &publication,
        "reference:writing",
        ScopeKind::Reference,
        "writing",
    );
    assert_scope(
        &publication,
        "outline:thesis/intro",
        ScopeKind::Outline,
        "thesis/intro",
    );
    assert_scope(
        &publication,
        "reference:thesis/intro",
        ScopeKind::Reference,
        "thesis/intro",
    );
    assert_scope(
        &publication,
        "bibliography:thesis/intro",
        ScopeKind::Bibliography,
        "thesis/intro",
    );
    assert_scope(
        &publication,
        "nav:thesis/intro",
        ScopeKind::Nav,
        "thesis/intro",
    );

    assert_property(
        &publication,
        "index",
        "title",
        Value::String("My Publication".to_string()),
    );
    assert_property(
        &publication,
        "writing",
        "title",
        Value::String("Writing".to_string()),
    );
    assert_property(
        &publication,
        "writing/blog-1",
        "title",
        Value::String("Blog 1".to_string()),
    );
    assert_property(
        &publication,
        "writing/blog-2",
        "title",
        Value::String("Blog 2".to_string()),
    );
    assert_property(
        &publication,
        "thesis/intro",
        "title",
        Value::String("Thesis".to_string()),
    );
    assert_property(
        &publication,
        "writing/blog-1",
        "citation",
        Value::TypstLabel("cite".to_string()),
    );
    assert_property(
        &publication,
        "thesis/ch-1",
        "citation",
        Value::TypstLabel("bib-ref".to_string()),
    );
    assert_property(&publication, "index", "nav.suppressed", Value::Bool(true));
    assert_property(&publication, "cv", "nav.suppressed", Value::Bool(true));

    assert_projection(
        &publication,
        "index",
        ProjectionKind::Outline,
        |projection| {
            projection
                .rendering_attributes
                .contains(&Attribute::new("depth", Value::Number(1)))
        },
    );
    assert_projection(
        &publication,
        "writing/blog-1",
        ProjectionKind::Bibliography,
        |projection| {
            projection.rendering_attributes.contains(&Attribute::new(
                "source",
                Value::String("works.yml".to_string()),
            )) && projection.rendering_attributes.contains(&Attribute::new(
                "scope",
                Value::String("current-page".to_string()),
            ))
        },
    );
    assert_projection(
        &publication,
        "writing/blog-2",
        ProjectionKind::Reference,
        |projection| {
            projection.rendering_attributes.contains(&Attribute::new(
                "target",
                Value::TypstLabel("figure-1".to_string()),
            ))
        },
    );
    assert_projection(
        &publication,
        "thesis/intro",
        ProjectionKind::Outline,
        |projection| {
            projection.rendering_attributes.contains(&Attribute::new(
                "title",
                Value::String("List of Figures".to_string()),
            )) && projection.rendering_attributes.contains(&Attribute::new(
                "target",
                Value::RawTypst("figure.where(kind: image)".to_string()),
            ))
        },
    );
    assert_projection(
        &publication,
        "thesis/bib",
        ProjectionKind::Bibliography,
        |projection| {
            projection.rendering_attributes.contains(&Attribute::new(
                "source",
                Value::String("thesis/bibliography.bib".to_string()),
            ))
        },
    );
    assert_projection(
        &publication,
        "index",
        ProjectionKind::Navigation,
        |projection| {
            projection.suppression
                == ProjectionSuppression::Suppressed {
                    reason: "property(nav.suppressed)".to_string(),
                }
        },
    );
    assert_projection(
        &publication,
        "cv",
        ProjectionKind::Navigation,
        |projection| {
            projection.suppression
                == ProjectionSuppression::Suppressed {
                    reason: "property(nav.suppressed)".to_string(),
                }
        },
    );

    assert_eq!(
        report.warnings,
        vec![ParseWarning::unreachable_typ_file("drafts/unreachable.typ")]
    );
}

fn assert_children(publication: &Publication, node_id: &str, expected: &[&str]) {
    let node = publication.node(&NodeId::from(node_id)).unwrap();
    let actual: Vec<_> = node.children.iter().map(NodeId::as_str).collect();
    assert_eq!(actual, expected, "children for {node_id}");
}

fn assert_scope(publication: &Publication, scope_id: &str, kind: ScopeKind, root_node: &str) {
    let scope = publication.scope(&ScopeId::from(scope_id)).unwrap();
    assert_eq!(scope.kind, kind);
    assert_eq!(scope.root_node, NodeId::from(root_node));
}

fn assert_property(publication: &Publication, node_id: &str, key: &str, expected_value: Value) {
    assert!(
        publication.properties.iter().any(|property| {
            property.owning_node == NodeId::from(node_id)
                && property.key == key
                && property.value == expected_value
        }),
        "missing property {key}={expected_value:?} on {node_id}"
    );
}

fn assert_projection(
    publication: &Publication,
    node_id: &str,
    kind: ProjectionKind,
    predicate: impl Fn(&Projection) -> bool,
) {
    assert!(
        publication.projections.iter().any(|projection| {
            projection.origin_node == NodeId::from(node_id)
                && projection.kind == kind
                && predicate(projection)
        }),
        "missing projection {kind:?} on {node_id}"
    );
}
