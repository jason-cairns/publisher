from __future__ import annotations

from pathlib import Path

from lxml import html
import pytest

from publication_site.cli import main


def test_transform_rejects_reserved_marker_missing_target(tmp_path):
    case = _case(tmp_path)
    case.write_html("index.typ", "<h1>Home</h1><publication-graph-link>Broken</publication-graph-link>")

    with pytest.raises(ValueError, match="publication-graph-link in index.typ missing data-target"):
        case.transform("index.typ")


def test_transform_rejects_unknown_reserved_marker(tmp_path):
    case = _case(tmp_path)
    case.write_html("index.typ", "<h1>Home</h1><publication-graph-note>authored?</publication-graph-note>")

    with pytest.raises(ValueError, match="unknown publication-graph marker publication-graph-note in index.typ"):
        case.transform("index.typ")


def test_transform_preserves_authored_publication_elements(tmp_path):
    case = _case(tmp_path)
    case.write_html("index.typ", "<h1>Home</h1><p><publication-note>Keep me</publication-note></p>")

    result = case.transform("index.typ")

    assert result == 0
    document = html.fromstring((case.out_dir / "index.html").read_text(encoding="utf-8"))
    notes = document.xpath("//publication-note")
    assert len(notes) == 1
    assert notes[0].text_content() == "Keep me"


def _case(tmp_path: Path) -> "_TransformCase":
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")

    return _TransformCase(src_dir, html_dir, out_dir, pdf_typ)


class _TransformCase:
    def __init__(self, src_dir: Path, html_dir: Path, out_dir: Path, pdf_typ: Path) -> None:
        self.src_dir = src_dir
        self.html_dir = html_dir
        self.out_dir = out_dir
        self.pdf_typ = pdf_typ

    def write_html(self, source: str, body: str) -> None:
        html_path = self.html_dir / f"{source.removesuffix('.typ')}.html"
        html_path.parent.mkdir(parents=True, exist_ok=True)
        html_path.write_text(
            f"<!DOCTYPE html><html><body>{body}</body></html>",
            encoding="utf-8",
        )

    def transform(self, *sources: str) -> int:
        return main(
            [
                "transform",
                "--src",
                str(self.src_dir),
                "--html",
                str(self.html_dir),
                "--out",
                str(self.out_dir),
                "--pdf-typ",
                str(self.pdf_typ),
                "--root",
                "index.typ",
                *sources,
            ]
        )
