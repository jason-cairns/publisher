use publisher::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
    assert_no_property(&publication, "nav.suppressed");

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
                    reason: "publisher.nav.suppress()".to_string(),
                }
                && has_attr(projection, "scope", Value::String("nav:index".to_string()))
        },
    );
    assert_projection(
        &publication,
        "writing",
        ProjectionKind::Navigation,
        |projection| {
            projection.suppression == ProjectionSuppression::NotSuppressed
                && has_attr(projection, "scope", Value::String("nav:index".to_string()))
        },
    );
    assert_nav_scopes(
        &publication,
        "thesis/bib",
        &["nav:index", "nav:thesis/intro"],
    );
    assert_projection(
        &publication,
        "cv",
        ProjectionKind::Navigation,
        |projection| {
            projection.suppression
                == ProjectionSuppression::Suppressed {
                    reason: "publisher.nav.suppress()".to_string(),
                }
                && has_attr(projection, "scope", Value::String("nav:index".to_string()))
        },
    );

    assert_eq!(
        report.warnings,
        vec![ParseWarning::unreachable_typ_file("drafts/unreachable.typ")]
    );
}

#[test]
fn parse_prd003_fixture_decodes_publish_edges_and_scope_metadata() {
    let publication = parse_publication("examples/prd003_discovery_site/index.typ").unwrap();

    let source_paths: Vec<_> = publication
        .nodes
        .iter()
        .map(|node| node.source_path.as_str())
        .collect();
    assert_eq!(
        source_paths,
        vec![
            "index.typ",
            "writing.typ",
            "writing/blog-1.typ",
            "writing/blog-2.typ",
            "thesis/intro.typ",
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
    assert_children(&publication, "thesis/intro", &["thesis/ch-1"]);
    assert!(
        publication.node(&NodeId::from("summary")).is_none(),
        "literal includes must not become routed published sources"
    );

    assert_publication_scope(&publication, "home", "index", None, &["nav"]);
    assert_publication_scope(
        &publication,
        "writing",
        "writing",
        Some("Writing"),
        &["nav"],
    );
    assert_publication_scope(
        &publication,
        "thesis",
        "thesis/intro",
        Some("Thesis"),
        &["nav", "bibliography"],
    );
    assert_publication_scope(&publication, "cv", "cv", Some("CV"), &[]);
}

fn assert_children(publication: &Publication, node_id: &str, expected: &[&str]) {
    let node = publication.node(&NodeId::from(node_id)).unwrap();
    let actual: Vec<_> = node.children.iter().map(NodeId::as_str).collect();
    assert_eq!(actual, expected, "children for {node_id}");
}

fn assert_publication_scope(
    publication: &Publication,
    scope_id: &str,
    root_node: &str,
    expected_title: Option<&str>,
    expected_tags: &[&str],
) {
    let scope = publication.scope(&ScopeId::from(scope_id)).unwrap();
    assert_eq!(scope.kind, ScopeKind::Publication);
    assert_eq!(scope.root_node, NodeId::from(root_node));
    assert_eq!(scope.name.as_deref(), expected_title);
    let actual_tags = scope
        .attributes
        .iter()
        .filter(|attribute| attribute.key == "tag")
        .map(|attribute| match &attribute.value {
            Value::String(value) => value.as_str(),
            other => panic!("expected string tag value, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_tags, expected_tags, "tags for {scope_id}");
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

fn assert_no_property(publication: &Publication, key: &str) {
    assert!(
        !publication
            .properties
            .iter()
            .any(|property| property.key == key),
        "unexpected property {key}"
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

fn assert_nav_scopes(publication: &Publication, node_id: &str, expected_scopes: &[&str]) {
    let actual = publication
        .projections
        .iter()
        .filter(|projection| {
            projection.origin_node == NodeId::from(node_id)
                && projection.kind == ProjectionKind::Navigation
        })
        .map(|projection| {
            projection
                .rendering_attributes
                .iter()
                .find(|attribute| attribute.key == "scope")
                .map(|attribute| attribute.value.clone())
                .unwrap()
        })
        .collect::<Vec<_>>();
    let expected = expected_scopes
        .iter()
        .map(|scope| Value::String((*scope).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "nav scopes for {node_id}");
}

fn has_attr(projection: &Projection, key: &str, value: Value) -> bool {
    projection
        .rendering_attributes
        .contains(&Attribute::new(key, value))
}

#[test]
fn malformed_publisher_markers_are_preserved_as_warnings() {
    let fixture = TestFixture::new("malformed-publisher-marker");
    fixture.write(
        "index.typ",
        r#"#import "/publisher.typ"
= Root
#metadata((kind: "child")) <publisher-marker>
"#,
    );
    fixture.write(
        "unused.typ",
        r#"#import "/publisher.typ"
= Unused
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();
    let report = publication.validate();

    assert!(report.errors.is_empty());
    assert_eq!(
        report.warnings,
        vec![
            ParseWarning {
                source_path: Some("index.typ".to_string()),
                kind: ParseWarningKind::UnsupportedPublisherCall,
                message: "unsupported publisher marker preserved for review: missing required field \"path\""
                    .to_string(),
            },
            ParseWarning::unreachable_typ_file("unused.typ"),
        ]
    );
}

#[test]
fn bare_nav_suppress_is_ignored_for_this_milestone() {
    let fixture = TestFixture::new("bare-nav-suppress");
    fixture.write("nav.typ", "#let suppress() = []\n");
    fixture.write(
        "index.typ",
        r#"#import "nav.typ"
= Root
#nav.suppress()
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();
    let report = publication.validate();

    assert!(report.errors.is_empty());
    assert!(report.warnings.is_empty());
    assert!(
        !publication
            .properties
            .iter()
            .any(|property| property.key == "nav.suppressed")
    );
    assert!(
        !publication
            .projections
            .iter()
            .any(|projection| projection.kind == ProjectionKind::Navigation)
    );
}

#[test]
fn named_scope_preserves_stable_id_and_authored_metadata() {
    let fixture = TestFixture::new("named-scope");
    fixture.write(
        "index.typ",
        r#"#import "/publisher.typ"
= Root
#publisher.scope(kind: "outline", name: "essays")
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();

    let scope = publication.scope(&ScopeId::from("outline:index")).unwrap();
    assert_eq!(scope.kind, ScopeKind::Outline);
    assert_eq!(scope.root_node, NodeId::from("index"));
    assert_eq!(scope.name.as_deref(), Some("essays"));
}

#[test]
fn publisher_aliases_and_wrappers_emit_parser_markers() {
    let fixture = TestFixture::new("publisher-alias-wrapper");
    fixture.write(
        "index.typ",
        r#"#import "/publisher.typ"
= Root
#let p = publisher
#let add-child(path) = p.child(path)
#add-child("section.typ")
#p.scope(kind: "nav")
"#,
    );
    fixture.write(
        "section.typ",
        r#"#import "/publisher.typ"
= Section
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();

    assert_children(&publication, "index", &["section"]);
    assert_scope(&publication, "nav:index", ScopeKind::Nav, "index");
    assert_nav_scopes(&publication, "section", &["nav:index"]);
}

#[test]
fn stacked_nav_scopes_create_stacked_navigation_projections() {
    let fixture = TestFixture::new("stacked-nav");
    fixture.write(
        "index.typ",
        r#"#import "/publisher.typ"
= Root
#publisher.child("section.typ")
#publisher.scope(kind: "nav")
"#,
    );
    fixture.write(
        "section.typ",
        r#"#import "/publisher.typ"
= Section
#publisher.child("leaf.typ")
#publisher.scope(kind: "nav")
"#,
    );
    fixture.write(
        "leaf.typ",
        r#"#import "/publisher.typ"
= Leaf
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();

    assert_nav_scopes(&publication, "leaf", &["nav:index", "nav:section"]);
}

#[test]
fn publisher_nav_suppress_suppresses_all_nav_projections_on_page_only() {
    let fixture = TestFixture::new("nav-suppress-page-only");
    fixture.write(
        "index.typ",
        r#"#import "/publisher.typ"
= Root
#publisher.child("section.typ")
#publisher.scope(kind: "nav")
"#,
    );
    fixture.write(
        "section.typ",
        r#"#import "/publisher.typ"
= Section
#publisher.child("leaf.typ")
#publisher.scope(kind: "nav")
#publisher.nav.suppress()
"#,
    );
    fixture.write(
        "leaf.typ",
        r#"#import "/publisher.typ"
= Leaf
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();

    let section_nav = nav_projections(&publication, "section");
    assert_eq!(section_nav.len(), 2);
    assert!(section_nav.iter().all(|projection| {
        projection.suppression
            == ProjectionSuppression::Suppressed {
                reason: "publisher.nav.suppress()".to_string(),
            }
    }));

    let leaf_nav = nav_projections(&publication, "leaf");
    assert_eq!(leaf_nav.len(), 2);
    assert!(
        leaf_nav
            .iter()
            .all(|projection| projection.suppression == ProjectionSuppression::NotSuppressed)
    );
    assert_no_property(&publication, "nav.suppressed");
}

fn nav_projections<'a>(publication: &'a Publication, node_id: &str) -> Vec<&'a Projection> {
    publication
        .projections
        .iter()
        .filter(|projection| {
            projection.origin_node == NodeId::from(node_id)
                && projection.kind == ProjectionKind::Navigation
        })
        .collect()
}

struct TestFixture {
    root: PathBuf,
}

impl TestFixture {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("publisher-{name}-{unique}"));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self { root };
        fixture.write(
            "publisher.typ",
            include_str!("../examples/api_sketch_site/publisher.typ"),
        );
        fixture.write(
            "publisher-nav.typ",
            include_str!("../examples/api_sketch_site/publisher-nav.typ"),
        );
        fixture
    }

    fn path(&self, rel_path: &str) -> PathBuf {
        self.root.join(rel_path)
    }

    fn write(&self, rel_path: &str, contents: &str) {
        let path = self.path(rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
