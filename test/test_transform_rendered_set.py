from __future__ import annotations

from publication_site.cli import main


def test_transform_drops_unreachable_candidates_from_output_and_pdf(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    (html_dir / "index.html").write_text(
        '<!DOCTYPE html><html><body><h1>Home</h1><publication-graph-publish data-target="child.typ">Child</publication-graph-publish></body></html>',
        encoding="utf-8",
    )
    (html_dir / "child.html").write_text(
        "<!DOCTYPE html><html><body><h1>Child</h1><p>Owned.</p></body></html>",
        encoding="utf-8",
    )
    (html_dir / "orphan.html").write_text(
        "<!DOCTYPE html><html><body><h1>Orphan</h1><p>Unowned.</p></body></html>",
        encoding="utf-8",
    )

    result = main(
        [
            "transform",
            "--src",
            str(src_dir),
            "--html",
            str(html_dir),
            "--out",
            str(out_dir),
            "--pdf-typ",
            str(pdf_typ),
            "--root",
            "index.typ",
            "index.typ",
            "child.typ",
            "orphan.typ",
        ]
    )

    assert result == 0
    assert (out_dir / "index.html").exists()
    assert (out_dir / "child" / "index.html").exists()
    assert not (out_dir / "orphan" / "index.html").exists()
    assert pdf_typ.read_text(encoding="utf-8") == (
        '#import "../src/_publication.typ": publication\n'
        "\n"
        '#publication("index.typ")[\n'
        '  #include "../src/index.typ"\n'
        "]\n\n"
        '#publication("child.typ")[\n'
        '  #include "../src/child.typ"\n'
        "]\n\n"
    )


def test_transform_ignores_unreachable_owners_when_building_navigation(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    (html_dir / "index.html").write_text(
        '<!DOCTYPE html><html><body><h1>Home</h1><publication-graph-publish data-target="child.typ">Child</publication-graph-publish></body></html>',
        encoding="utf-8",
    )
    (html_dir / "child.html").write_text(
        "<!DOCTYPE html><html><body><h1>Child</h1><p>Owned by the rendered root.</p></body></html>",
        encoding="utf-8",
    )
    (html_dir / "orphan.html").write_text(
        '<!DOCTYPE html><html><body><h1>Orphan</h1><publication-graph-entry data-target="child.typ"></publication-graph-entry></body></html>',
        encoding="utf-8",
    )

    result = main(
        [
            "transform",
            "--src",
            str(src_dir),
            "--html",
            str(html_dir),
            "--out",
            str(out_dir),
            "--pdf-typ",
            str(pdf_typ),
            "--root",
            "index.typ",
            "index.typ",
            "child.typ",
            "orphan.typ",
        ]
    )

    assert result == 0
    child_html = (out_dir / "child" / "index.html").read_text(encoding="utf-8")
    assert '<a href="/child/" aria-current="page">Child</a>' in child_html
    assert not (out_dir / "orphan" / "index.html").exists()
