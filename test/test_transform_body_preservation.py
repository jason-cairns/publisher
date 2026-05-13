from __future__ import annotations

from pathlib import Path

from lxml import html

from publication_site.cli import main


def test_transform_preserves_body_text_tails_and_inline_marker_contents(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    _write_html(
        html_dir,
        "index.typ",
        (
            "<h2><em>Intro</em></h2>"
            '<publication-graph-entry data-target="child.typ"></publication-graph-entry>'
            '<p>Before <publication-graph-link data-target="child.typ">'
            "<em>Child</em>"
            "</publication-graph-link> after.</p>"
        ),
    )
    _write_html(html_dir, "child.typ", "<h2>Child</h2><p>Owned.</p>")

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
        ]
    )

    assert result == 0
    document = html.fromstring((out_dir / "index.html").read_text(encoding="utf-8"))

    assert document.xpath("string(//title)") == "Intro - cair.nz"
    assert document.xpath("string(//main/h2)") == "Intro"

    paragraph = document.xpath("//main/p")[0]
    anchor = paragraph.xpath("./a")[0]

    assert paragraph.text == "Before "
    assert anchor.get("href") == "/child/"
    assert anchor.text is None
    assert anchor.xpath("./em")[0].text == "Child"
    assert anchor.tail == " after."
    assert paragraph.xpath("string()") == "Before Child after."
    assert not document.xpath("//*[starts-with(local-name(), 'publication-graph-')]")


def test_transform_stitches_paragraphs_split_by_inline_publication_markers(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    _write_html(
        html_dir,
        "index.typ",
        (
            "<h1>Home</h1>"
            '<publication-graph-entry data-target="child.typ"></publication-graph-entry>'
            "<p>Before </p>"
            '<publication-graph-link data-target="child.typ">Child</publication-graph-link>'
            "<p> after.</p>"
            "<p>Label </p>"
            '<publication-graph-ref data-label="child-note">note</publication-graph-ref>'
            "<p> tail.</p>"
        ),
    )
    _write_html(
        html_dir,
        "child.typ",
        '<h1>Child</h1><publication-graph-label data-label="child-note"></publication-graph-label>',
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
        ]
    )

    assert result == 0
    document = html.fromstring((out_dir / "index.html").read_text(encoding="utf-8"))
    paragraphs = document.xpath("//main/p")

    assert len(paragraphs) == 2
    assert paragraphs[0].xpath("string()") == "Before Child after."
    assert paragraphs[0].xpath("./a")[0].get("href") == "/child/"
    assert paragraphs[1].xpath("string()") == "Label note tail."
    assert paragraphs[1].xpath("./a")[0].get("href") == "/child/#child-note"


def test_transform_does_not_stitch_across_standalone_publication_labels(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    _write_html(
        html_dir,
        "index.typ",
        (
            "<h1>Home</h1>"
            "<p>First paragraph.</p>"
            '<publication-graph-label data-label="note"></publication-graph-label>'
            "<p>Second paragraph.</p>"
        ),
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
        ]
    )

    assert result == 0
    document = html.fromstring((out_dir / "index.html").read_text(encoding="utf-8"))
    paragraphs = document.xpath("//main/p")

    assert len(paragraphs) == 2
    assert paragraphs[0].xpath("string()") == "First paragraph."
    assert paragraphs[1].xpath("string()") == "Second paragraph."
    assert document.xpath("count(//main/span[@id='note'])") == 1.0


def _write_html(html_dir: Path, source: str, body: str) -> None:
    html_path = html_dir / f"{source.removesuffix('.typ')}.html"
    html_path.parent.mkdir(parents=True, exist_ok=True)
    html_path.write_text(
        f"<!DOCTYPE html><html><body>{body}</body></html>",
        encoding="utf-8",
    )
