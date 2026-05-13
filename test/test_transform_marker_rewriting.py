from __future__ import annotations

from pathlib import Path

import pytest

from publication_site.cli import main


def test_transform_rewrites_markers_to_public_html(tmp_path):
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
            '<p><publication-graph-link data-target="child.typ">Child</publication-graph-link></p>'
            '<publication-graph-label data-label="root-note"></publication-graph-label>'
            '<p><publication-graph-ref data-label="root-note">root note</publication-graph-ref></p>'
            '<p><publication-graph-ref data-label="child-note">child note</publication-graph-ref></p>'
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
    index_html = (out_dir / "index.html").read_text(encoding="utf-8")
    child_html = (out_dir / "child" / "index.html").read_text(encoding="utf-8")
    public_html = index_html + child_html

    assert '<a href="/child/">Child</a>' in index_html
    assert '<span id="root-note"></span>' in index_html
    assert '<a href="#root-note">root note</a>' in index_html
    assert '<a href="/child/#child-note">child note</a>' in index_html
    assert '<span id="child-note"></span>' in child_html
    assert "<publication-graph-" not in public_html


def test_transform_rejects_missing_label_refs(tmp_path):
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
        '<h1>Home</h1><publication-graph-ref data-label="missing-note">missing</publication-graph-ref>',
    )

    with pytest.raises(ValueError, match="label missing-note is not rendered"):
        main(
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


def _write_html(html_dir: Path, source: str, body: str) -> None:
    html_path = html_dir / f"{source.removesuffix('.typ')}.html"
    html_path.parent.mkdir(parents=True, exist_ok=True)
    html_path.write_text(
        f"<!DOCTYPE html><html><body>{body}</body></html>",
        encoding="utf-8",
    )
