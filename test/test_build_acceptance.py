from __future__ import annotations

from pathlib import Path
import shutil

from publication_site.cli import main
from publication_site.typst_compile import compile_typst_pdf

from conftest import BuiltSite, TransformCase


SITE_SOURCES = [
    "index.typ",
    "colophon.typ",
    "posts/foo.typ",
    "thesis.typ",
    "thesis/chapter.typ",
    "writing.typ",
]


def test_build_writes_expected_site_artifacts(built_site: BuiltSite) -> None:
    for path in [
        built_site.html_dir / "index.html",
        built_site.html_dir / "writing.html",
        built_site.html_dir / "posts" / "foo.html",
        built_site.html_dir / "thesis.html",
        built_site.html_dir / "thesis" / "chapter.html",
        built_site.public_dir / "index.html",
        built_site.public_dir / "writing" / "index.html",
        built_site.public_dir / "posts" / "foo" / "index.html",
        built_site.public_dir / "thesis" / "index.html",
        built_site.public_dir / "thesis" / "chapter" / "index.html",
        built_site.public_dir / "colophon" / "index.html",
        built_site.pdf_typ,
        built_site.pdf_out,
    ]:
        assert path.is_file()
        assert path.stat().st_size > 0


def test_unified_pdf_assembly_follows_ownership_preorder(built_site: BuiltSite) -> None:
    assert _publication_order(built_site.pdf_typ) == [
        "index.typ",
        "writing.typ",
        "posts/foo.typ",
        "thesis.typ",
        "thesis/chapter.typ",
        "colophon.typ",
    ]


def test_pdf_assembly_paths_are_relative_to_assembly_location(
    tmp_path: Path,
    built_site: BuiltSite,
) -> None:
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "html"
    public_dir = tmp_path / "public"
    pdf_typ = tmp_path / "nested" / "site.typ"
    pdf_out = tmp_path / "site.pdf"

    shutil.copytree(built_site.src_dir, src_dir)
    shutil.copytree(built_site.html_dir, html_dir)

    result = main(
        [
            "transform",
            "--src",
            str(src_dir),
            "--html",
            str(html_dir),
            "--out",
            str(public_dir),
            "--pdf-typ",
            str(pdf_typ),
            "--root",
            "index.typ",
            *SITE_SOURCES,
        ]
    )

    assert result == 0
    assembly = pdf_typ.read_text(encoding="utf-8")
    assert '#import "../src/_publication.typ": publication' in assembly
    assert '#include "../src/index.typ"' in assembly

    compile_typst_pdf(pdf_typ, pdf_out, root=tmp_path)
    assert pdf_out.is_file()
    assert pdf_out.stat().st_size > 0


def test_intermediate_html_carries_marker_protocol(built_site: BuiltSite) -> None:
    index_html = (built_site.html_dir / "index.html").read_text(encoding="utf-8")

    assert "<body>" in index_html
    assert (
        '<publication-graph-publish data-target="writing.typ">Writing</publication-graph-publish>'
        in index_html
    )
    assert '<publication-graph-entry data-target="colophon.typ"></publication-graph-entry>' in index_html
    assert '<publication-graph-label data-label="root-note"></publication-graph-label>' in index_html
    assert '<publication-graph-ref data-label="foo-note">foo note</publication-graph-ref>' in index_html


def test_final_html_has_site_chrome_routes_and_publication_indexes(built_site: BuiltSite) -> None:
    public = built_site.public_dir

    index_html = (public / "index.html").read_text(encoding="utf-8")
    writing_html = (public / "writing" / "index.html").read_text(encoding="utf-8")
    foo_html = (public / "posts" / "foo" / "index.html").read_text(encoding="utf-8")
    thesis_html = (public / "thesis" / "index.html").read_text(encoding="utf-8")
    chapter_html = (public / "thesis" / "chapter" / "index.html").read_text(encoding="utf-8")
    colophon_html = (public / "colophon" / "index.html").read_text(encoding="utf-8")

    assert "<main>" in index_html
    assert "standalone Typst publication" in index_html
    assert "<nav>" in index_html
    assert '<a href="/writing/">Writing</a>' in index_html
    assert '<a href="/thesis/">Thesis</a>' in index_html
    assert '<a href="/writing/" aria-current="page">Writing</a>' in writing_html
    assert "anonymous ownership edge" in foo_html
    assert '<a href="/thesis/" aria-current="page">Thesis</a>' in thesis_html
    assert '<a href="/thesis/chapter/">Chapter</a>' in thesis_html
    assert '<a href="/thesis/chapter/" aria-current="page">Chapter</a>' in chapter_html
    assert "public without becoming" in colophon_html
    assert '<link rel="stylesheet" href="/writing.css">' in writing_html
    assert "<link rel=\"stylesheet\"" not in index_html


def test_publication_links_and_labels_resolve_to_routes_and_fragments(built_site: BuiltSite) -> None:
    public = built_site.public_dir

    index_html = (public / "index.html").read_text(encoding="utf-8")
    writing_html = (public / "writing" / "index.html").read_text(encoding="utf-8")
    foo_html = (public / "posts" / "foo" / "index.html").read_text(encoding="utf-8")
    chapter_html = (public / "thesis" / "chapter" / "index.html").read_text(encoding="utf-8")
    colophon_html = (public / "colophon" / "index.html").read_text(encoding="utf-8")

    assert '<a href="/colophon/">colophon</a>' in index_html
    assert '<a href="/posts/foo/">Foo</a>' in writing_html
    assert '<a href="/writing/">Writing</a>' in foo_html
    assert '<a href="/thesis/">Thesis</a>' in chapter_html
    assert '<a href="/">home</a>' in colophon_html
    assert '<span id="root-note"></span>' in index_html
    assert '<a href="#root-note">root note</a>' in index_html
    assert '<span id="foo-note"></span>' in foo_html
    assert '<a href="#foo-note">own note</a>' in foo_html
    assert '<a href="/posts/foo/#foo-note">foo note</a>' in index_html


def test_public_html_has_no_internal_markers_or_javascript(built_site: BuiltSite) -> None:
    public_html = _read_tree_text(built_site.public_dir, "*.html")

    assert "<publication-graph-" not in public_html
    assert "<script" not in public_html.lower()


def test_protocol_names_are_generic_not_site_branded(repo_root: Path) -> None:
    branded_protocol_name = "cairn" + "z"
    implementation_text = "".join(
        path.read_text(encoding="utf-8")
        for directory in ["src_py", "src", "test"]
        for path in (repo_root / directory).rglob("*")
        if path.is_file() and path.suffix in {".py", ".typ", ".sh", ".html"}
    )

    assert branded_protocol_name not in implementation_text.lower()


def test_entry_ownership_does_not_create_index_items(built_site: BuiltSite) -> None:
    index_html = (built_site.public_dir / "index.html").read_text(encoding="utf-8")
    writing_html = (built_site.public_dir / "writing" / "index.html").read_text(encoding="utf-8")
    foo_html = (built_site.public_dir / "posts" / "foo" / "index.html").read_text(encoding="utf-8")
    chapter_html = (
        built_site.public_dir / "thesis" / "chapter" / "index.html"
    ).read_text(encoding="utf-8")

    assert '<a href="/colophon/">colophon</a></li>' not in index_html
    assert writing_html.count("<nav>") == 1
    assert foo_html.count("<nav>") == 1
    assert chapter_html.count("<nav>") == 2


def test_representative_pages_match_golden_snapshots(built_site: BuiltSite) -> None:
    for relative_path in [
        Path("index.html"),
        Path("posts/foo/index.html"),
        Path("thesis/chapter/index.html"),
        Path("colophon/index.html"),
    ]:
        assert (built_site.public_dir / relative_path).read_text(encoding="utf-8") == (
            built_site.repo_root / "test" / "golden" / "public" / relative_path
        ).read_text(encoding="utf-8")


def test_each_publication_compiles_independently(built_site: BuiltSite, tmp_path: Path) -> None:
    pdf_out = tmp_path / "index.pdf"

    compile_typst_pdf(built_site.src_dir / "index.typ", pdf_out, root=built_site.src_dir)

    assert pdf_out.is_file()
    assert pdf_out.stat().st_size > 0


def test_sources_outside_ownership_tree_are_not_published(
    transform_case: TransformCase,
    built_site: BuiltSite,
) -> None:
    transform_case.copy_intermediate_html_from(built_site.html_dir)
    transform_case.write_html("orphan.typ", "<h2>Orphan</h2>")

    result = transform_case.transform(*SITE_SOURCES, "orphan.typ")

    assert result == 0
    assert (transform_case.out_dir / "index.html").is_file()
    assert (transform_case.out_dir / "colophon" / "index.html").is_file()
    assert (transform_case.out_dir / "posts" / "foo" / "index.html").is_file()
    assert (transform_case.out_dir / "thesis" / "index.html").is_file()
    assert (transform_case.out_dir / "thesis" / "chapter" / "index.html").is_file()
    assert (transform_case.out_dir / "writing" / "index.html").is_file()
    assert not (transform_case.out_dir / "orphan" / "index.html").exists()
    assert '#publication("colophon.typ")' in transform_case.pdf_typ.read_text(encoding="utf-8")
    assert '#publication("orphan.typ")' not in transform_case.pdf_typ.read_text(encoding="utf-8")


def test_inline_heading_markup_can_supply_publication_title(transform_case: TransformCase) -> None:
    transform_case.write_html("index.typ", "<h2><em>Intro</em></h2><p>Body.</p>")

    result = transform_case.transform("index.typ")

    assert result == 0
    rendered = (transform_case.out_dir / "index.html").read_text(encoding="utf-8")
    assert "<title>Intro - cair.nz</title>" in rendered
    assert "<h2><em>Intro</em></h2>" in rendered


def test_root_publication_is_configurable(transform_case: TransformCase) -> None:
    transform_case.write_html(
        "writing.typ",
        (
            '<publication-graph-publish data-target="index.typ">Home</publication-graph-publish>'
            '<p><publication-graph-link data-target="index.typ">Home</publication-graph-link></p>'
        ),
    )
    transform_case.write_html(
        "index.typ",
        '<p><publication-graph-link data-target="writing.typ">Root</publication-graph-link></p>',
    )

    result = transform_case.transform("writing.typ", "index.typ", root="writing.typ")

    assert result == 0
    root_html = (transform_case.out_dir / "index.html").read_text(encoding="utf-8")
    child_html = (transform_case.out_dir / "index" / "index.html").read_text(encoding="utf-8")
    assert '<a href="/index/">Home</a>' in root_html
    assert '<a href="/">Root</a>' in child_html
    assert _publication_order(transform_case.pdf_typ) == ["writing.typ", "index.typ"]


def test_unrendered_labels_do_not_satisfy_label_refs(
    transform_case: TransformCase,
    built_site: BuiltSite,
) -> None:
    transform_case.copy_intermediate_html_from(built_site.html_dir)
    transform_case.write_html(
        "orphan.typ",
        '<h2>Orphan</h2><publication-graph-label data-label="orphan-note"></publication-graph-label>',
    )
    with (transform_case.html_dir / "index.html").open("a", encoding="utf-8") as handle:
        handle.write('<publication-graph-ref data-label="orphan-note">orphan note</publication-graph-ref>')

    try:
        transform_case.transform(*SITE_SOURCES, "orphan.typ")
    except ValueError as error:
        assert str(error) == "label orphan-note is not rendered"
    else:
        raise AssertionError("label references must point to labels in rendered publications")


def test_duplicate_rendered_labels_fail_validation(
    transform_case: TransformCase,
    built_site: BuiltSite,
) -> None:
    transform_case.copy_intermediate_html_from(built_site.html_dir)
    with (transform_case.html_dir / "colophon.html").open("a", encoding="utf-8") as handle:
        handle.write('<publication-graph-label data-label="root-note"></publication-graph-label>')

    try:
        transform_case.transform(*SITE_SOURCES)
    except ValueError as error:
        assert str(error) == "duplicate label root-note"
    else:
        raise AssertionError("duplicate rendered site-global publication labels must fail validation")


def _publication_order(pdf_typ: Path) -> list[str]:
    return [
        line.removeprefix('#publication("').split('"', 1)[0]
        for line in pdf_typ.read_text(encoding="utf-8").splitlines()
        if line.startswith('#publication("')
    ]


def _read_tree_text(directory: Path, pattern: str) -> str:
    return "".join(path.read_text(encoding="utf-8") for path in directory.rglob(pattern))
