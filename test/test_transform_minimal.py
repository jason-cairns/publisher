from __future__ import annotations

from publication_site.cli import main


def test_transform_writes_root_html_and_pdf_assembly(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    (html_dir / "index.html").write_text(
        "<!DOCTYPE html><html><body><h1>Home</h1><p>Hello.</p></body></html>",
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
    assert (out_dir / "index.html").read_text(encoding="utf-8") == (
        "<!doctype html>\n"
        '<html lang="en">\n'
        "  <head>\n"
        '    <meta charset="utf-8">\n'
        '    <meta name="viewport" content="width=device-width, initial-scale=1">\n'
        "    <title>Home - cair.nz</title>\n"
        "  </head>\n"
        "  <body>\n"
        '    <header><a href="/">cair.nz</a></header>\n'
        "    <main>\n"
        "      <h1>Home</h1><p>Hello.</p>\n"
        "    </main>\n"
        "  </body>\n"
        "</html>\n"
    )
    assert pdf_typ.read_text(encoding="utf-8") == (
        '#import "../src/_publication.typ": publication\n'
        "\n"
        '#publication("index.typ")[\n'
        '  #include "../src/index.typ"\n'
        "]\n\n"
    )


def test_transform_serializes_page_title_entities(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    (html_dir / "index.html").write_text(
        "<!DOCTYPE html><html><body><h1>AT&amp;T &lt;Site&gt;</h1></body></html>",
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
    assert "<title>AT&amp;T &lt;Site&gt; - cair.nz</title>" in (out_dir / "index.html").read_text(
        encoding="utf-8"
    )


def test_transform_escapes_decoded_title_text(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    (html_dir / "index.html").write_text(
        "<!DOCTYPE html><html><body><h1>Rock &amp; Roll &lt;Again&gt;</h1></body></html>",
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
    rendered = (out_dir / "index.html").read_text(encoding="utf-8")

    assert "    <title>Rock &amp; Roll &lt;Again&gt; - cair.nz</title>\n" in rendered
    assert "<title>Rock & Roll <Again> - cair.nz</title>" not in rendered


def test_transform_supports_default_css_and_route_override(tmp_path):
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")
    (html_dir / "index.html").write_text(
        '<!DOCTYPE html><html><body><h1>Home</h1><publication-graph-publish data-target="writing.typ">Writing</publication-graph-publish></body></html>',
        encoding="utf-8",
    )
    (html_dir / "writing.html").write_text(
        '<!DOCTYPE html><html><body><h1>Writing</h1><publication-graph-stylesheet data-href="/writing.css"></publication-graph-stylesheet></body></html>',
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
            "--css",
            "/site.css",
            "index.typ",
            "writing.typ",
        ]
    )

    assert result == 0
    assert '<link rel="stylesheet" href="/site.css">' in (out_dir / "index.html").read_text(encoding="utf-8")
    assert '<link rel="stylesheet" href="/writing.css">' in (
        out_dir / "writing" / "index.html"
    ).read_text(encoding="utf-8")
