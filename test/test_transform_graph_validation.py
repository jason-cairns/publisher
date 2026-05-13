from __future__ import annotations

from pathlib import Path

import pytest

from publication_site.cli import main


def test_transform_rejects_invalid_rendered_publication_graphs(tmp_path):
    cases = [
        (
            "rendered owner publishes missing target",
            {
                "index.typ": '<h1>Home</h1><publication-graph-entry data-target="missing.typ"></publication-graph-entry>',
            },
            ["index.typ"],
            "unknown ownership target missing.typ",
        ),
        (
            "rendered publication has duplicate owners",
            {
                "index.typ": '<h1>Home</h1><publication-graph-entry data-target="first.typ"></publication-graph-entry><publication-graph-entry data-target="second.typ"></publication-graph-entry>',
                "first.typ": '<h1>First</h1><publication-graph-entry data-target="shared.typ"></publication-graph-entry>',
                "second.typ": '<h1>Second</h1><publication-graph-entry data-target="shared.typ"></publication-graph-entry>',
                "shared.typ": "<h1>Shared</h1>",
            },
            ["index.typ", "first.typ", "second.typ", "shared.typ"],
            "multiple owners for shared.typ",
        ),
        (
            "root is owned",
            {
                "index.typ": '<h1>Home</h1><publication-graph-entry data-target="child.typ"></publication-graph-entry>',
                "child.typ": '<h1>Child</h1><publication-graph-entry data-target="index.typ"></publication-graph-entry>',
            },
            ["index.typ", "child.typ"],
            "root publication index.typ must not be owned",
        ),
        (
            "rendered publication links to known unrendered source",
            {
                "index.typ": '<h1>Home</h1><publication-graph-link data-target="orphan.typ">Orphan</publication-graph-link>',
                "orphan.typ": "<h1>Orphan</h1>",
            },
            ["index.typ", "orphan.typ"],
            "link target orphan.typ is not rendered",
        ),
    ]

    for name, documents, sources, expected in cases:
        work_dir = tmp_path / name.replace(" ", "-")
        src_dir = work_dir / "src"
        html_dir = work_dir / "build" / "html"
        out_dir = work_dir / "public"
        pdf_typ = work_dir / "build" / "site.typ"

        src_dir.mkdir(parents=True)
        html_dir.mkdir(parents=True)
        (src_dir / "_publication.typ").write_text("", encoding="utf-8")
        for source, body in documents.items():
            _write_html(html_dir, source, body)

        with pytest.raises(ValueError, match=expected):
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
                    *sources,
                ]
            )


def _write_html(html_dir: Path, source: str, body: str) -> None:
    html_path = html_dir / f"{source.removesuffix('.typ')}.html"
    html_path.parent.mkdir(parents=True, exist_ok=True)
    html_path.write_text(
        f"<!DOCTYPE html><html><body>{body}</body></html>",
        encoding="utf-8",
    )
