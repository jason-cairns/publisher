from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import shutil

import pytest

from publication_site.cli import main


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class BuiltSite:
    repo_root: Path
    src_dir: Path
    html_dir: Path
    public_dir: Path
    pdf_typ: Path
    pdf_out: Path


@pytest.fixture(scope="session")
def built_site(tmp_path_factory: pytest.TempPathFactory, repo_root: Path) -> BuiltSite:
    work_dir = tmp_path_factory.mktemp("built-site")
    src_dir = work_dir / "src"
    html_dir = work_dir / "build" / "html"
    public_dir = work_dir / "public"
    pdf_typ = work_dir / "build" / "site.typ"
    pdf_out = work_dir / "public" / "site.pdf"

    shutil.copytree(repo_root / "src", src_dir)

    result = main(
        [
            "build",
            "--src",
            str(src_dir),
            "--html",
            str(html_dir),
            "--out",
            str(public_dir),
            "--pdf-typ",
            str(pdf_typ),
            "--pdf-out",
            str(pdf_out),
            "--root",
            "index.typ",
        ]
    )

    assert result == 0
    return BuiltSite(repo_root, src_dir, html_dir, public_dir, pdf_typ, pdf_out)


@dataclass(frozen=True)
class TransformCase:
    src_dir: Path
    html_dir: Path
    out_dir: Path
    pdf_typ: Path

    def write_html(self, source: str, body: str) -> None:
        html_path = self.html_dir / f"{source.removesuffix('.typ')}.html"
        html_path.parent.mkdir(parents=True, exist_ok=True)
        html_path.write_text(
            f"<!DOCTYPE html><html><body>{body}</body></html>",
            encoding="utf-8",
        )

    def copy_intermediate_html_from(self, source_dir: Path) -> None:
        if self.html_dir.exists():
            shutil.rmtree(self.html_dir)
        shutil.copytree(source_dir, self.html_dir)

    def transform(self, *sources: str, root: str = "index.typ") -> int:
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
                root,
                *sources,
            ]
        )


@pytest.fixture
def transform_case(tmp_path: Path) -> TransformCase:
    src_dir = tmp_path / "src"
    html_dir = tmp_path / "build" / "html"
    out_dir = tmp_path / "public"
    pdf_typ = tmp_path / "build" / "site.typ"

    src_dir.mkdir()
    html_dir.mkdir(parents=True)
    (src_dir / "_publication.typ").write_text("", encoding="utf-8")

    return TransformCase(src_dir, html_dir, out_dir, pdf_typ)
