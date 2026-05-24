use publisher::*;

#[test]
fn render_assembly_preserves_api_sketch_nodes_and_authored_source() {
    let publication = parse_publication("examples/api_sketch_site/index.typ").unwrap();
    let report = publication.validate();
    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    let document = build_assembly(&publication);
    let assembly = document.to_typst();

    assert_eq!(document.nodes.len(), 8);
    assert!(assembly.contains("= API Sketch Publication"));
    assert!(assembly.contains("Source: `index.typ`"));
    assert!(assembly.contains("Children: `writing.typ`, `thesis/intro.typ`, `cv.typ`"));
    assert!(assembly.contains("Source: `writing/blog-1.typ`"));
    assert!(assembly.contains("Parent: `writing.typ`"));
    assert!(assembly.contains("Source: `thesis/bib.typ`"));
    assert!(assembly.contains("Source: `cv.typ`"));
    assert!(!assembly.contains("drafts/unreachable.typ"));

    assert!(assembly.contains("Hello, welcome to my publication!"));
    assert!(assembly.contains("This is my blog. I can have images,"));
    assert!(assembly.contains("#publisher.bibliography(\"works.yml\", scope: \"current-page\")"));
    assert!(assembly.contains("I can reference #publisher.ref(<figure-1>) from the thesis"));
}

#[test]
fn render_assembly_includes_projection_placeholder_detail_without_query_evaluation() {
    let publication = parse_publication("examples/api_sketch_site/index.typ").unwrap();
    let assembly = render_assembly(&publication);

    assert!(
        assembly.contains(
            "Generated from the typed publication model. Query evaluation is not implemented in this assembly."
        ),
        "assembly should make query evaluation non-goal visible"
    );
    assert!(assembly.contains("projection-placeholder"));

    assert!(assembly.contains("kind: outline"));
    assert!(assembly.contains("origin: index.typ"));
    assert!(assembly.contains("rendering: depth=1"));
    assert!(assembly.contains("query-selection: nodes explicit"));
    assert!(assembly.contains("query-search: nearest-scope defaulted"));

    assert!(assembly.contains("kind: navigation"));
    assert!(assembly.contains("suppression: suppressed=true reason=publisher.nav.suppress()"));
    assert!(assembly.contains("rendering: scope=\"nav:index\", scope-root=\"index\""));
    assert!(assembly.contains("query-search: named-scope(nav:index) explicit"));
    assert!(assembly.contains("query-filter: scope.id == \"nav:index\" explicit"));

    assert!(assembly.contains("kind: bibliography"));
    assert!(assembly.contains("rendering: source=\"works.yml\", scope=\"current-page\""));
    assert!(assembly.contains("query-selection: properties explicit"));
    assert!(assembly.contains("query-filter: property.key == \"citation\" explicit"));

    assert!(assembly.contains("kind: reference"));
    assert!(assembly.contains("rendering: target=<figure-1>"));
    assert!(assembly.contains("query-selection: resolved-target explicit"));
    assert!(assembly.contains("query-filter: target == <figure-1> explicit"));
}
