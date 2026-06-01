use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use publisher::*;

const FIXTURE_ROOT: &str = "examples/discovery_site";
const FIXTURE_INDEX: &str = "examples/discovery_site/index.typ";

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}

fn install_typst_package(data_dir: &PathBuf) {
    let install = Command::new("sh")
        .arg("scripts/install-typst-package.sh")
        .env("PUBLISHER_TYPST_DATA_DIR", data_dir)
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "install failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );
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
fn installed_typst_package_imports_work_through_cli() {
    let temp = temp_dir("publisher-package-cli");
    let data_dir = temp.join("data");
    let site_dir = temp.join("site");
    let out_dir = temp.join("out");
    fs::create_dir_all(site_dir.join("posts")).unwrap();

    install_typst_package(&data_dir);

    fs::write(
        site_dir.join("index.typ"),
        r#"#import "@local/publisher:0.1.0": scope, publish

= Home

#publish("posts/a.typ")
#scope("home")
"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("posts/a.typ"),
        r#"#import "@local/publisher:0.1.0": scope

= Post A

#scope("post")
"#,
    )
    .unwrap();

    let render = Command::new(env!("CARGO_BIN_EXE_publisher"))
        .arg("render")
        .arg("--root")
        .arg(site_dir.join("index.typ"))
        .arg("--to")
        .arg("html")
        .arg("--out")
        .arg(&out_dir)
        .env("PUBLISHER_TYPST_DATA_DIR", &data_dir)
        .output()
        .unwrap();
    assert!(
        render.status.success(),
        "render failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&render.stdout),
        String::from_utf8_lossy(&render.stderr)
    );
    assert!(out_dir.join("index.html").is_file());
    assert!(out_dir.join("posts/a.html").is_file());
}

#[test]
fn cli_accepts_bare_root_filename_from_publication_directory() {
    let temp = temp_dir("publisher-bare-root-cli");
    let data_dir = temp.join("data");
    let site_dir = temp.join("site");
    let out_dir = temp.join("out");
    fs::create_dir_all(&site_dir).unwrap();
    install_typst_package(&data_dir);

    fs::write(
        site_dir.join("index.typ"),
        r#"#import "@local/publisher:0.1.0": scope

= Home

#scope("home")
"#,
    )
    .unwrap();

    let render = Command::new(env!("CARGO_BIN_EXE_publisher"))
        .current_dir(&site_dir)
        .arg("render")
        .arg("--root")
        .arg("index.typ")
        .arg("--to")
        .arg("html")
        .arg("--out")
        .arg(&out_dir)
        .env("PUBLISHER_TYPST_DATA_DIR", &data_dir)
        .output()
        .unwrap();
    assert!(
        render.status.success(),
        "render failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&render.stdout),
        String::from_utf8_lossy(&render.stderr)
    );
    assert!(out_dir.join("index.html").is_file());
}

#[test]
fn marker_evaluation_accepts_real_svg_assets() {
    let temp = temp_dir("publisher-svg-asset-cli");
    let data_dir = temp.join("data");
    let site_dir = temp.join("site");
    let out_dir = temp.join("out");
    fs::create_dir_all(site_dir.join("assets")).unwrap();
    install_typst_package(&data_dir);

    fs::write(
        site_dir.join("index.typ"),
        r#"#import "@local/publisher:0.1.0": scope

= Home

#scope("home")
#figure(image("assets/Smiley.svg"))
"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("assets/Smiley.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="gold"/></svg>"#,
    )
    .unwrap();

    let render = Command::new(env!("CARGO_BIN_EXE_publisher"))
        .arg("render")
        .arg("--root")
        .arg(site_dir.join("index.typ"))
        .arg("--to")
        .arg("html")
        .arg("--out")
        .arg(&out_dir)
        .env("PUBLISHER_TYPST_DATA_DIR", &data_dir)
        .output()
        .unwrap();
    assert!(
        render.status.success(),
        "render failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&render.stdout),
        String::from_utf8_lossy(&render.stderr)
    );
    assert!(out_dir.join("index.html").is_file());
}

#[test]
fn whole_publication_pdf_uses_root_stem_as_artifact_name() {
    let temp = temp_dir("publisher-pdf-root-stem-cli");
    let data_dir = temp.join("data");
    let site_dir = temp.join("site");
    let out_dir = temp.join("out");
    fs::create_dir_all(&site_dir).unwrap();
    install_typst_package(&data_dir);

    fs::write(
        site_dir.join("index.typ"),
        r#"#import "@local/publisher:0.1.0": scope

= Index

#scope("index")
"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("cv.typ"),
        r#"#import "@local/publisher:0.1.0": scope

= CV

#scope("cv")
"#,
    )
    .unwrap();

    for root in ["index.typ", "cv.typ"] {
        let render = Command::new(env!("CARGO_BIN_EXE_publisher"))
            .arg("render")
            .arg("--root")
            .arg(site_dir.join(root))
            .arg("--to")
            .arg("pdf")
            .arg("--out")
            .arg(&out_dir)
            .env("PUBLISHER_TYPST_DATA_DIR", &data_dir)
            .output()
            .unwrap();
        assert!(
            render.status.success(),
            "render failed for {root}:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&render.stdout),
            String::from_utf8_lossy(&render.stderr)
        );
    }

    assert!(out_dir.join("index.pdf").is_file());
    assert!(out_dir.join("index.typ").is_file());
    assert!(out_dir.join("cv.pdf").is_file());
    assert!(out_dir.join("cv.typ").is_file());
    assert!(!out_dir.join("publication.pdf").exists());
}

#[test]
fn css_file_payloads_are_inherited_into_html_head() {
    let temp = temp_dir("publisher-css-payload-cli");
    let data_dir = temp.join("data");
    let site_dir = temp.join("site");
    let out_dir = temp.join("out");
    fs::create_dir_all(site_dir.join("styles")).unwrap();
    fs::create_dir_all(site_dir.join("writing/styles")).unwrap();
    install_typst_package(&data_dir);

    fs::write(
        site_dir.join("index.typ"),
        r#"#import "@local/publisher:0.1.0": scope, publish, css

= Home

#scope("home")
#css("styles/site.css")
#publish("writing/index.typ")
"#,
    )
    .unwrap();
    fs::write(
        site_dir.join("writing/index.typ"),
        r#"#import "@local/publisher:0.1.0": scope, css

= Writing

#scope("writing")
#css("styles/writing.css")
"#,
    )
    .unwrap();
    fs::write(site_dir.join("styles/site.css"), "body { color: #111; }\n").unwrap();
    fs::write(
        site_dir.join("writing/styles/writing.css"),
        "body { color: #222; }\n",
    )
    .unwrap();

    let render = Command::new(env!("CARGO_BIN_EXE_publisher"))
        .arg("render")
        .arg("--root")
        .arg(site_dir.join("index.typ"))
        .arg("--to")
        .arg("html")
        .arg("--out")
        .arg(&out_dir)
        .env("PUBLISHER_TYPST_DATA_DIR", &data_dir)
        .output()
        .unwrap();
    assert!(
        render.status.success(),
        "render failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&render.stdout),
        String::from_utf8_lossy(&render.stderr)
    );

    let index = fs::read_to_string(out_dir.join("index.html")).unwrap();
    assert!(index.contains(r#"data-publisher-css="styles/site.css""#));
    assert!(index.contains("body { color: #111; }"));
    assert!(!index.contains("body { color: #222; }"));

    let writing = fs::read_to_string(out_dir.join("writing/index.html")).unwrap();
    let site_at = writing.find("body { color: #111; }").unwrap();
    let writing_at = writing.find("body { color: #222; }").unwrap();
    assert!(
        site_at < writing_at,
        "ancestor CSS should precede nested CSS"
    );
    assert!(writing.contains(r#"data-publisher-css="styles/writing.css""#));
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
