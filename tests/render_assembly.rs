use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use publisher::*;

const FIXTURE_ROOT: &str = "examples/prd003_discovery_site";
const FIXTURE_INDEX: &str = "examples/prd003_discovery_site/index.typ";

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}

fn options(output_dir: PathBuf, artifact_name: &str) -> RenderOptions {
    RenderOptions {
        source_root: FIXTURE_ROOT.into(),
        output_dir,
        artifact_name: artifact_name.to_string(),
    }
}

fn fixture() -> Publication {
    let publication = parse_publication(FIXTURE_INDEX).unwrap();
    let report = publication.validate();
    assert!(
        report.errors.is_empty(),
        "expected valid fixture, got {:#?}",
        report.errors
    );
    publication
}

#[test]
fn validation_errors_block_rendering() {
    let mut publication = Publication::new(Node::new("index", "index.typ"));
    publication.add_node(Node::new("dup", "index.typ")); // duplicate source path

    let options = options(temp_dir("publisher-invalid"), "invalid");
    let error = render_target(
        &publication,
        &RenderTarget::WholePublication,
        Edition::Html,
        &options,
    )
    .unwrap_err();
    assert!(matches!(error, RenderError::ValidationFailed { .. }));
}

#[test]
fn unknown_scope_is_rejected() {
    let publication = fixture();
    let options = options(temp_dir("publisher-unknown-scope"), "missing");
    let error = render_target(
        &publication,
        &RenderTarget::NamedScope(ScopeId::from("does-not-exist")),
        Edition::Pdf,
        &options,
    )
    .unwrap_err();
    assert!(matches!(error, RenderError::UnknownScope { .. }));
}

#[test]
fn html_edition_writes_one_page_per_source() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-html-pages");
    let artifacts = render_target(
        &publication,
        &RenderTarget::WholePublication,
        Edition::Html,
        &options(output_dir.clone(), "publication"),
    )
    .unwrap();

    let mut routes: Vec<_> = artifacts
        .html_paths
        .iter()
        .map(|path| path.strip_prefix(&output_dir).unwrap().to_path_buf())
        .collect();
    routes.sort();
    let mut expected: Vec<PathBuf> = [
        "index.html",
        "writing.html",
        "writing/blog-1.html",
        "writing/blog-2.html",
        "thesis/intro.html",
        "thesis/ch-1.html",
        "cv.html",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    expected.sort();
    assert_eq!(routes, expected);
    assert!(artifacts.pdf_path.is_none());
    for path in &artifacts.html_paths {
        assert!(path.is_file(), "missing rendered page {}", path.display());
    }
}

#[test]
fn html_cross_source_reference_links_without_placeholder() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-html-xref");
    render_target(
        &publication,
        &RenderTarget::WholePublication,
        Edition::Html,
        &options(output_dir.clone(), "publication"),
    )
    .unwrap();

    // blog-2 references @thesis-main, which lives in thesis/ch-1.typ.
    let blog_2 = fs::read_to_string(output_dir.join("writing/blog-2.html")).unwrap();
    assert!(
        blog_2.contains("thesis/ch-1.html#thesis-main"),
        "expected cross-source link, got:\n{blog_2}"
    );
    // Real display text with titled-scope context, not a raw placeholder.
    assert!(blog_2.contains("Chapter One, Thesis"));
    assert!(!blog_2.contains("placeholder"));
}

#[test]
fn html_page_renders_single_bibliography_of_cited_works() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-html-bib");
    render_target(
        &publication,
        &RenderTarget::WholePublication,
        Edition::Html,
        &options(output_dir.clone(), "publication"),
    )
    .unwrap();

    // thesis/intro.typ cites @book only; its page lists just that work.
    let intro = fs::read_to_string(output_dir.join("thesis/intro.html")).unwrap();
    assert!(intro.contains("Thesis Source"), "cited work missing");
    assert!(
        !intro.contains("Chapter Source"),
        "uncited work should not appear"
    );
    assert!(
        !intro.contains("Web Reference"),
        "uncited work should not appear"
    );
}

#[test]
fn whole_publication_pdf_compiles_with_one_consolidated_bibliography() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-pdf-whole");
    let artifacts = render_target(
        &publication,
        &RenderTarget::WholePublication,
        Edition::Pdf,
        &options(output_dir.clone(), "publication"),
    )
    .unwrap();

    let pdf_path = artifacts.pdf_path.expect("pdf path");
    assert!(pdf_path.is_file());
    assert!(fs::metadata(&pdf_path).unwrap().len() > 0);
    assert!(artifacts.html_paths.is_empty());

    // The generated entrypoint carries exactly one bibliography call over the
    // shared works.yml, even though four sources each authored one.
    let entrypoint = fs::read_to_string(artifacts.entrypoint_path.expect("entrypoint")).unwrap();
    assert_eq!(entrypoint.matches("#bibliography(").count(), 1);
    assert!(entrypoint.contains("#bibliography(\"works.yml\")"));
}

#[test]
fn named_scope_pdf_includes_only_scope_sources() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-pdf-thesis");
    let artifacts = render_target(
        &publication,
        &RenderTarget::NamedScope(ScopeId::from("thesis")),
        Edition::Pdf,
        &options(output_dir.clone(), "thesis"),
    )
    .unwrap();

    let pdf_path = artifacts.pdf_path.expect("pdf path");
    assert!(pdf_path.is_file());
    assert!(fs::metadata(&pdf_path).unwrap().len() > 0);

    let entrypoint = fs::read_to_string(artifacts.entrypoint_path.expect("entrypoint")).unwrap();
    assert!(entrypoint.contains("#include \"thesis/intro.typ\""));
    assert!(entrypoint.contains("#include \"thesis/ch-1.typ\""));
    assert!(!entrypoint.contains("#include \"writing.typ\""));
    assert!(!entrypoint.contains("#include \"cv.typ\""));
}

#[test]
fn scope_union_pdf_dedupes_and_orders_by_graph() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-pdf-union");
    let artifacts = render_target(
        &publication,
        &RenderTarget::ScopeUnion(vec![ScopeId::from("writing"), ScopeId::from("thesis")]),
        Edition::Pdf,
        &options(output_dir.clone(), "writing+thesis"),
    )
    .unwrap();

    let entrypoint = fs::read_to_string(artifacts.entrypoint_path.expect("entrypoint")).unwrap();
    // writing and thesis sources are present, cv (outside both) is not.
    assert!(entrypoint.contains("#include \"writing.typ\""));
    assert!(entrypoint.contains("#include \"writing/blog-1.typ\""));
    assert!(entrypoint.contains("#include \"thesis/intro.typ\""));
    assert!(entrypoint.contains("#include \"thesis/ch-1.typ\""));
    assert!(!entrypoint.contains("#include \"cv.typ\""));
    // Each source appears exactly once even though scopes overlap.
    assert_eq!(
        entrypoint.matches("#include \"thesis/ch-1.typ\"").count(),
        1
    );
    // writing.typ precedes thesis/intro.typ in publication graph order.
    let writing_at = entrypoint.find("#include \"writing.typ\"").unwrap();
    let thesis_at = entrypoint.find("#include \"thesis/intro.typ\"").unwrap();
    assert!(writing_at < thesis_at);
}

#[test]
fn pdf_outline_is_bounded_to_active_scope() {
    let publication = fixture();
    let output_dir = temp_dir("publisher-pdf-outline");
    let artifacts = render_target(
        &publication,
        &RenderTarget::NamedScope(ScopeId::from("thesis")),
        Edition::Pdf,
        &options(output_dir.clone(), "thesis"),
    )
    .unwrap();

    // thesis/intro.typ authored `#outline(target: heading.where(level: 1))`;
    // the entrypoint brackets the thesis region so the outline is bounded.
    let entrypoint = fs::read_to_string(artifacts.entrypoint_path.expect("entrypoint")).unwrap();
    assert!(entrypoint.contains("<scope-start-thesis>"));
    assert!(entrypoint.contains("<scope-end-thesis>"));
}
