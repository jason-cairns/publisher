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
    assert!(output_dir.join("typst/index.typ").is_file());
    assert!(output_dir.join("typst/writing.typ").is_file());
    assert!(output_dir.join("typst/writing/blog-1.typ").is_file());

    let pdf_source = fs::read_to_string(output_dir.join("api-sketch.typ")).unwrap();
    assert!(pdf_source.contains("#include \"typst-pdf/index.typ\""));
    assert!(pdf_source.contains("#include \"typst-pdf/writing.typ\""));
    assert!(pdf_source.contains("#include \"typst-pdf/writing/blog-1.typ\""));
    assert!(pdf_source.contains("#pagebreak()"));
    assert!(!pdf_source.contains("Node id:"));
    assert!(!pdf_source.contains("Authored source:"));

    let index = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index.contains("Hello, welcome to my publication!"));
    assert!(index.contains("projection-placeholder"));
    assert!(index.contains("kind: outline"));
    assert!(index.contains("kind: navigation"));
    assert!(!index.contains("Node id:"));
    assert!(!index.contains("Authored source:"));

    let blog_1 = fs::read_to_string(output_dir.join("writing/blog-1.html")).unwrap();
    assert!(blog_1.contains("This is my blog. I can have images"));
    assert!(blog_1.contains("kind: navigation"));
    assert!(blog_1.contains("kind: bibliography"));
    assert!(blog_1.contains("@cite placeholder"));

    let writing = fs::read_to_string(output_dir.join("writing.html")).unwrap();
    assert!(writing.contains("Writing"));
    assert!(writing.contains("kind: navigation"));
    assert!(writing.contains("scope=\"nav:index\""));
    assert!(writing.contains("kind: outline"));

    let blog_2 = fs::read_to_string(output_dir.join("writing/blog-2.html")).unwrap();
    assert!(blog_2.contains("I can reference"));
    assert!(blog_2.contains("kind: navigation"));
    assert!(blog_2.contains("kind: reference"));

    let cv = fs::read_to_string(output_dir.join("cv.html")).unwrap();
    assert!(cv.contains("about me"));
    assert!(cv.contains("kind: navigation"));
}

#[test]
fn render_publication_lowers_prd003_outline_to_active_scope_headings() {
    let fixture = TestFixture::new("publisher-prd003-outline");
    fixture.write(
        "publisher.typ",
        include_str!("../examples/prd003_discovery_site/publisher.typ"),
    );
    fixture.write(
        "index.typ",
        r#"#import "publisher.typ": scope, publish
= Home
#publish("writing.typ")
#publish("cv.typ")
#scope("home")
"#,
    );
    fixture.write(
        "writing.typ",
        r#"#import "publisher.typ": scope, publish
= Writing
#scope("writing", title: [Writing])
#publish("writing/blog-1.typ")

#outline(target: heading.where(level: 1))
"#,
    );
    fixture.write(
        "writing/blog-1.typ",
        r#"#import "../publisher.typ": scope
= Blog One
"#,
    );
    fixture.write(
        "cv.typ",
        r#"#import "publisher.typ": scope
= CV
#scope("cv", title: [CV])
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();
    let report = publication.validate();
    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    let output_dir = temp_render_dir("publisher-prd003-outline-render");
    let options = RenderOptions {
        source_root: fixture.root.clone(),
        output_dir: output_dir.clone(),
        artifact_name: "prd003-outline".to_string(),
    };

    render_publication(&publication, &options).unwrap();

    let generated_writing = fs::read_to_string(output_dir.join("typst/writing.typ")).unwrap();
    assert!(!generated_writing.contains("#outline("));
    assert!(generated_writing.contains("Scope-local outline"));
    assert!(generated_writing.contains("scope: writing"));
    assert!(generated_writing.contains("- Writing"));
    assert!(generated_writing.contains("- Blog One"));
    assert!(!generated_writing.contains("- CV"));

    let writing_html = fs::read_to_string(output_dir.join("writing.html")).unwrap();
    assert!(writing_html.contains("Scope-local outline"));
    assert!(writing_html.contains("scope: writing"));
    assert!(writing_html.contains("Blog One"));
    assert!(!writing_html.contains("CV"));
}

#[test]
fn render_publication_preserves_source_bibliography_for_html_and_lowers_for_combined_pdf() {
    let fixture = TestFixture::new("publisher-prd003-bibliography");
    fixture.write(
        "publisher.typ",
        include_str!("../examples/prd003_discovery_site/publisher.typ"),
    );
    fixture.write(
        "works.yml",
        r#"web:
  type: Web
  title: Web Reference
  author: Doe, Jane
  url: https://example.com/web

article:
  type: Article
  title: Article Reference
  author: Roe, Sam
  parent:
    type: Periodical
    title: Journal
"#,
    );
    fixture.write(
        "index.typ",
        r#"#import "publisher.typ": scope, publish
= Home
#publish("writing/blog-1.typ")
#publish("thesis/ch-1.typ")
#scope("home")
"#,
    );
    fixture.write(
        "writing/blog-1.typ",
        r#"#import "../publisher.typ": scope
= Blog One
#scope("writing", title: [Writing])

This article cites @web.

#bibliography("../works.yml")
"#,
    );
    fixture.write(
        "thesis/ch-1.typ",
        r#"#import "../publisher.typ": scope
= Chapter One
#scope("thesis", title: [Thesis])

This chapter cites @article.

#bibliography("../works.yml")
"#,
    );

    let publication = parse_publication(fixture.path("index.typ")).unwrap();
    let report = publication.validate();
    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );

    let output_dir = temp_render_dir("publisher-prd003-bibliography-render");
    let options = RenderOptions {
        source_root: fixture.root.clone(),
        output_dir: output_dir.clone(),
        artifact_name: "prd003-bibliography".to_string(),
    };

    render_publication(&publication, &options).unwrap();

    let html_blog_source = fs::read_to_string(output_dir.join("typst/writing/blog-1.typ")).unwrap();
    assert!(html_blog_source.contains("#bibliography(\"../works.yml\")"));
    assert_eq!(html_blog_source.matches("#bibliography(").count(), 1);

    let pdf_blog_source =
        fs::read_to_string(output_dir.join("typst-pdf/writing/blog-1.typ")).unwrap();
    assert!(!pdf_blog_source.contains("#bibliography("));
    assert!(pdf_blog_source.contains("Scoped bibliography"));
    assert!(pdf_blog_source.contains("scope: writing"));
    assert!(pdf_blog_source.contains("source: ../works.yml"));

    let pdf_chapter_source =
        fs::read_to_string(output_dir.join("typst-pdf/thesis/ch-1.typ")).unwrap();
    assert!(!pdf_chapter_source.contains("#bibliography("));
    assert!(pdf_chapter_source.contains("Scoped bibliography"));
    assert!(pdf_chapter_source.contains("scope: thesis"));

    let pdf_source = fs::read_to_string(output_dir.join("prd003-bibliography.typ")).unwrap();
    assert!(pdf_source.contains("#include \"typst-pdf/writing/blog-1.typ\""));
    assert!(pdf_source.contains("#include \"typst-pdf/thesis/ch-1.typ\""));
}

fn temp_render_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
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
        let root = std::env::temp_dir().join(format!("{name}-{unique}"));
        fs::create_dir_all(&root).unwrap();
        Self { root }
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
