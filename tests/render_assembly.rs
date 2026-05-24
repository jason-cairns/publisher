use publisher::*;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[test]
fn render_publication_rejects_validation_errors_before_export() {
    let mut publication = parse_publication("examples/api_sketch_site/index.typ").unwrap();
    publication.queries.clear();

    let options = RenderOptions {
        source_root: "examples/api_sketch_site".into(),
        output_dir: std::env::temp_dir().join("publisher-render-invalid"),
        artifact_name: "invalid".to_string(),
    };

    let error = render_publication(&publication, &options).unwrap_err();
    assert!(matches!(error, RenderError::ValidationFailed { .. }));
}

#[test]
fn render_publication_writes_one_html_page_per_reachable_node() {
    let publication = parse_publication("examples/api_sketch_site/index.typ").unwrap();
    let output_dir = temp_render_dir("publisher-render-pages");
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("api-sketch.html"), "stale combined html").unwrap();

    let options = RenderOptions {
        source_root: "examples/api_sketch_site".into(),
        output_dir: output_dir.clone(),
        artifact_name: "api-sketch".to_string(),
    };

    let artifacts = render_publication(&publication, &options).unwrap();
    let expected_html_paths = vec![
        output_dir.join("index.html"),
        output_dir.join("writing.html"),
        output_dir.join("writing/blog-1.html"),
        output_dir.join("writing/blog-2.html"),
        output_dir.join("thesis/intro.html"),
        output_dir.join("thesis/bib.html"),
        output_dir.join("thesis/ch-1.html"),
        output_dir.join("cv.html"),
    ];

    assert_eq!(artifacts.html_paths, expected_html_paths);
    assert!(artifacts.typst_path.is_file());
    assert!(artifacts.pdf_path.is_file());
    assert!(!output_dir.join("api-sketch.html").exists());

    let index = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index.contains("Hello, welcome to my publication!"));
    assert!(index.contains("projection-placeholder"));
    assert!(index.contains("kind: outline"));
    assert!(index.contains("kind: navigation"));
    assert!(!index.contains("Node id:"));
    assert!(!index.contains("Authored source:"));

    let blog_1 = fs::read_to_string(output_dir.join("writing/blog-1.html")).unwrap();
    assert!(blog_1.contains("This is my blog. I can have images"));
    assert!(blog_1.contains("kind: bibliography"));
    assert!(blog_1.contains("@cite placeholder"));

    let blog_2 = fs::read_to_string(output_dir.join("writing/blog-2.html")).unwrap();
    assert!(blog_2.contains("I can reference"));
    assert!(blog_2.contains("kind: reference"));

    let cv = fs::read_to_string(output_dir.join("cv.html")).unwrap();
    assert!(cv.contains("about me"));
    assert!(cv.contains("kind: navigation"));
}

fn temp_render_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}
