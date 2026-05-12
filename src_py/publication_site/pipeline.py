from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path

import networkx as nx
from lxml import html


def build_site(*_args: object, **_kwargs: object) -> None:
    raise NotImplementedError("site build is not implemented yet")


def transform_site(
    src_dir: Path,
    html_dir: Path,
    out_dir: Path,
    pdf_typ: Path,
    root: str,
    sources: list[str],
) -> None:
    documents = {source: _read_document(html_dir, source) for source in sources}
    ownership_edges = [
        edge
        for source, document in documents.items()
        for edge in _ownership_edges(source, document)
    ]
    link_edges = [
        edge
        for source, document in documents.items()
        for edge in _link_edges(source, document)
    ]
    rendered = _validated_ownership_preorder(root, sources, ownership_edges, link_edges)

    for source in rendered:
        document = documents[source]
        title = _title(document, source)
        out_path = _public_path(out_dir, root, source)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(_site_page(root, title, _body_html(document)), encoding="utf-8")

    pdf_typ.parent.mkdir(parents=True, exist_ok=True)
    pdf_typ.write_text(_pdf_typ(src_dir, pdf_typ, rendered), encoding="utf-8")


@dataclass(frozen=True)
class OwnershipEdge:
    owner: str
    target: str


@dataclass(frozen=True)
class LinkEdge:
    source: str
    target: str


def _read_document(html_dir: Path, source: str) -> html.HtmlElement:
    return html.fromstring((_html_path(html_dir, source)).read_text(encoding="utf-8"))


def _ownership_edges(source: str, document: html.HtmlElement) -> list[OwnershipEdge]:
    markers = document.xpath("//publication-graph-publish | //publication-graph-entry")
    return [
        OwnershipEdge(source, target)
        for marker in markers
        if (target := marker.get("data-target")) is not None
    ]


def _link_edges(source: str, document: html.HtmlElement) -> list[LinkEdge]:
    markers = document.xpath("//publication-graph-link")
    return [
        LinkEdge(source, target)
        for marker in markers
        if (target := marker.get("data-target")) is not None
    ]


def _validated_ownership_preorder(
    root: str,
    sources: list[str],
    ownership_edges: list[OwnershipEdge],
    link_edges: list[LinkEdge],
) -> list[str]:
    source_set = set(sources)
    if root not in source_set:
        raise ValueError(f"root publication {root} is not a known source")

    graph = nx.DiGraph()
    graph.add_nodes_from(sources)
    graph.add_edges_from(
        (edge.owner, edge.target)
        for edge in ownership_edges
        if edge.owner in source_set and edge.target in source_set
    )
    rendered = list(nx.dfs_preorder_nodes(graph, root))
    rendered_set = set(rendered)

    for edge in ownership_edges:
        if edge.owner in rendered_set and edge.target not in source_set:
            raise ValueError(f"unknown ownership target {edge.target} from {edge.owner}")

    root_owners = [edge.owner for edge in ownership_edges if edge.target == root and edge.owner in rendered_set]
    if root_owners:
        raise ValueError(f"root publication {root} must not be owned")

    for source in rendered:
        owners = [owner for owner in graph.predecessors(source) if owner in rendered_set]
        if source != root and len(owners) != 1:
            raise ValueError(f"multiple owners for {source}: {', '.join(sorted(owners))}")

    for edge in link_edges:
        if edge.source in rendered_set and edge.target not in rendered_set:
            raise ValueError(f"link target {edge.target} is not rendered")

    return rendered


def _html_path(html_dir: Path, source: str) -> Path:
    return html_dir / f"{_source_stem(source)}.html"


def _public_path(out_dir: Path, root: str, source: str) -> Path:
    if source == root:
        return out_dir / "index.html"
    return out_dir / _source_stem(source) / "index.html"


def _source_stem(source: str) -> str:
    return source.removesuffix(".typ")


def _title(document: html.HtmlElement, source: str) -> str:
    heading = document.xpath("//body//*[self::h1 or self::h2 or self::h3 or self::h4 or self::h5 or self::h6][1]")
    if not heading:
        return source
    return heading[0].text_content()


def _body_html(document: html.HtmlElement) -> str:
    body = document.find("body")
    if body is None:
        return ""
    return "".join(html.tostring(child, encoding="unicode") for child in body).strip()


def _site_page(root: str, title: str, body: str) -> str:
    return (
        "<!doctype html>\n"
        '<html lang="en">\n'
        "  <head>\n"
        '    <meta charset="utf-8">\n'
        '    <meta name="viewport" content="width=device-width, initial-scale=1">\n'
        f"    <title>{title} - cair.nz</title>\n"
        "  </head>\n"
        "  <body>\n"
        f'    <header><a href="{_route_href(root, root)}">cair.nz</a></header>\n'
        "    <main>\n"
        f"      {body}\n"
        "    </main>\n"
        "  </body>\n"
        "</html>\n"
    )


def _route_href(root: str, source: str) -> str:
    if source == root:
        return "/"
    return f"/{_source_stem(source)}/"


def _pdf_typ(src_dir: Path, pdf_typ: Path, sources: list[str]) -> str:
    src_prefix = Path(os.path.relpath(src_dir, pdf_typ.parent)).as_posix()
    publications = "".join(
        f'#publication("{source}")[\n'
        f'  #include "{src_prefix}/{source}"\n'
        "]\n\n"
        for source in sources
    )
    return f'#import "{src_prefix}/_publication.typ": publication\n\n{publications}'
