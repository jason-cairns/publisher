from __future__ import annotations

from publication_site.typst_compile import compile_typst_html


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
