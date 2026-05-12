from __future__ import annotations

from publication_site.cli import main


def test_build_writes_intermediate_html_final_html_pdf_assembly_and_pdf(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"
    pdf_out = tmp_path / "public" / "site.pdf"

    src_dir.mkdir()
    (src_dir / "_publication.typ").write_text(
        '#let html_export = sys.inputs.at("render-target", default: "paged") == "html"\n'
        "#let publish(dest, body) = context {\n"
        "  if html_export {\n"
        '    html.elem("publication-graph-publish", attrs: ("data-target": dest), body)\n'
        "  } else {\n"
        "    body\n"
        "  }\n"
        "}\n"
        "#let publication(dest, body) = context [#body]\n",
        encoding="utf-8",
    )
    (src_dir / "index.typ").write_text(
        '#import "_publication.typ": publish\n'
        "= Home\n"
        "\n"
        "#publish(\"writing.typ\")[Writing]\n"
        "\n"
        "Welcome.\n",
        encoding="utf-8",
    )
    (src_dir / "writing.typ").write_text(
        "= Writing\n"
        "\n"
        "Body.\n",
        encoding="utf-8",
    )

    result = main(
        [
            "build",
            "--src",
            str(src_dir),
            "--html",
            str(html_dir),
            "--out",
            str(out_dir),
            "--pdf-typ",
            str(pdf_typ),
            "--pdf-out",
            str(pdf_out),
            "--root",
            "index.typ",
        ]
    )

    assert result == 0
    assert (html_dir / "index.html").read_text(encoding="utf-8").startswith("<!DOCTYPE html>")
    assert (html_dir / "writing.html").read_text(encoding="utf-8").startswith("<!DOCTYPE html>")
    assert (out_dir / "index.html").is_file()
    assert (out_dir / "writing" / "index.html").is_file()
    assert '#publication("index.typ")' in pdf_typ.read_text(encoding="utf-8")
    assert pdf_out.is_file()
    assert pdf_out.stat().st_size > 0
