from __future__ import annotations

from pathlib import Path

from publication_site.cli import main
from publication_site.typst_compile import compile_typst_html


PUBLICATION_HELPER = Path(__file__).resolve().parents[1] / "src" / "_publication.typ"


def test_typst_package_compiles_tiny_typst_fixture_to_html(tmp_path, monkeypatch):
    source = tmp_path / "tiny.typ"
    source.write_text(
        '#let render_target = sys.inputs.at("render-target", default: "paged")\n'
        '#if render_target == "html" [= Tiny HTML]\n',
        encoding="utf-8",
    )

    monkeypatch.setenv("TYPST_FEATURES", "html")

    html = compile_typst_html(
        source,
        root=tmp_path,
        sys_inputs={"render-target": "html"},
    )

    assert "<html>" in html
    assert "Tiny HTML" in html


def test_publication_stylesheet_macro_survives_typst_html_transform(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"
    source = src_dir / "index.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text(PUBLICATION_HELPER.read_text(encoding="utf-8"), encoding="utf-8")
    source.write_text(
        '#import "_publication.typ": publication-stylesheet\n'
        "\n"
        "= Styled\n"
        "\n"
        '#publication-stylesheet("/site.css")\n',
        encoding="utf-8",
    )

    (html_dir / "index.html").write_text(
        compile_typst_html(source, root=src_dir, sys_inputs={"render-target": "html"}),
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
        ]
    )

    assert result == 0
    assert '<link rel="stylesheet" href="/site.css">' in (out_dir / "index.html").read_text(encoding="utf-8")
