from __future__ import annotations

from contextlib import contextmanager
import os
from pathlib import Path

import typst


def compile_typst_html(
    source: Path,
    *,
    root: Path,
    sys_inputs: dict[str, str],
) -> str:
    with _typst_features("html"):
        html = typst.compile(
            source,
            root=root,
            format="html",
            sys_inputs=sys_inputs,
        )
    if isinstance(html, bytes):
        return html.decode("utf-8")
    return str(html)


def compile_typst_pdf(
    source: Path,
    output: Path,
    *,
    root: Path,
) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    typst.compile(source, output=output, root=root)


@contextmanager
def _typst_features(feature: str):
    original = os.environ.get("TYPST_FEATURES")
    features = [] if original is None else [item for item in original.split(",") if item]
    if feature not in features:
        features.append(feature)
    os.environ["TYPST_FEATURES"] = ",".join(features)
    try:
        yield
    finally:
        if original is None:
            os.environ.pop("TYPST_FEATURES", None)
        else:
            os.environ["TYPST_FEATURES"] = original
