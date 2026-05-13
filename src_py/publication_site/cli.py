from __future__ import annotations

import argparse
from pathlib import Path

from .pipeline import build_site, transform_site


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="site")
    subparsers = parser.add_subparsers(dest="command", required=True)

    build = subparsers.add_parser("build", help="Build final HTML and unified PDF.")
    build.add_argument("--src", default="src", type=Path)
    build.add_argument("--html", default="build/html", type=Path)
    build.add_argument("--out", default="public", type=Path)
    build.add_argument("--pdf-typ", default="build/site.typ", type=Path)
    build.add_argument("--pdf-out", default="public/site.pdf", type=Path)
    build.add_argument("--root", default="index.typ")
    build.add_argument("--css")

    transform = subparsers.add_parser(
        "transform",
        help="Transform existing intermediate HTML fixtures without compiling Typst.",
    )
    transform.add_argument("--src", required=True, type=Path)
    transform.add_argument("--html", required=True, type=Path)
    transform.add_argument("--out", required=True, type=Path)
    transform.add_argument("--pdf-typ", required=True, type=Path)
    transform.add_argument("--root", required=True)
    transform.add_argument("--css")
    transform.add_argument("sources", nargs="+")

    args = parser.parse_args(argv)

    if args.command == "build":
        build_site(args.src, args.html, args.out, args.pdf_typ, args.pdf_out, args.root, args.css)
    elif args.command == "transform":
        transform_site(args.src, args.html, args.out, args.pdf_typ, args.root, args.sources, args.css)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
