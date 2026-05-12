from __future__ import annotations

from pathlib import Path

import typst


def compile_typst_html(
    source: Path,
    *,
    root: Path,
    sys_inputs: dict[str, str],
) -> str:
    html = typst.compile(
        source,
        root=root,
        format="html",
        sys_inputs=sys_inputs,
    )
    if isinstance(html, bytes):
        return html.decode("utf-8")
    return str(html)
