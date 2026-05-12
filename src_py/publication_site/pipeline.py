from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
from html import escape
import os
from pathlib import Path
import shutil

import networkx as nx
from lxml import html

from .typst_compile import compile_typst_html, compile_typst_pdf


def build_site(
    src_dir: Path,
    html_dir: Path,
    out_dir: Path,
    pdf_typ: Path,
    pdf_out: Path,
    root: str,
) -> None:
    sources = _publication_sources(src_dir)
    shutil.rmtree(html_dir, ignore_errors=True)
    shutil.rmtree(out_dir, ignore_errors=True)

    for source in sources:
        html_path = _html_path(html_dir, source)
        html_path.parent.mkdir(parents=True, exist_ok=True)
        html_path.write_text(
            compile_typst_html(
                src_dir / source,
                root=src_dir,
                sys_inputs={"render-target": "html"},
            ),
            encoding="utf-8",
        )

    transform_site(src_dir, html_dir, out_dir, pdf_typ, root, sources)
    compile_typst_pdf(pdf_typ, pdf_out, root=_compile_root(src_dir, pdf_typ, pdf_out))


def transform_site(
    src_dir: Path,
    html_dir: Path,
    out_dir: Path,
    pdf_typ: Path,
    root: str,
    sources: list[str],
) -> None:
    documents = {source: _read_document(html_dir, source) for source in sources}
    for source, document in documents.items():
        _validate_reserved_markers(source, document)

    ownership_edges = [
        edge
        for source, document in documents.items()
        for edge in _ownership_edges(source, document)
    ]
    publication_indices = [
        item
        for source, document in documents.items()
        for item in _publication_index_items(source, document)
    ]
    link_edges = [
        edge
        for source, document in documents.items()
        for edge in _link_edges(source, document)
    ]
    label_defs = [
        label_def
        for source, document in documents.items()
        for label_def in _label_defs(source, document)
    ]
    label_refs = [
        label_ref
        for source, document in documents.items()
        for label_ref in _label_refs(source, document)
    ]
    rendered = _validated_ownership_preorder(root, sources, ownership_edges, link_edges)
    label_sources = _validated_label_sources(rendered, label_defs, label_refs)

    for source in rendered:
        document = documents[source]
        _rewrite_markers(document, root, source, label_sources)
        title = _title(document, source)
        nav_html = _navigation_html(root, source, rendered, ownership_edges, publication_indices)
        out_path = _public_path(out_dir, root, source)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(_site_page(root, title, nav_html, _body_html(document)), encoding="utf-8")

    pdf_typ.parent.mkdir(parents=True, exist_ok=True)
    pdf_typ.write_text(_pdf_typ(src_dir, pdf_typ, rendered), encoding="utf-8")


def _publication_sources(src_dir: Path) -> list[str]:
    return [
        source.relative_to(src_dir).as_posix()
        for source in sorted(src_dir.rglob("*.typ"))
        if not source.name.startswith("_")
    ]


def _compile_root(*paths: Path) -> Path:
    return Path(os.path.commonpath([path.resolve() for path in paths]))


@dataclass(frozen=True)
class OwnershipEdge:
    owner: str
    target: str


@dataclass(frozen=True)
class LinkEdge:
    source: str
    target: str


@dataclass(frozen=True)
class PublicationIndexItem:
    owner: str
    target: str
    title: str


@dataclass(frozen=True)
class LabelDef:
    source: str
    label: str


@dataclass(frozen=True)
class LabelRef:
    source: str
    label: str


_RESERVED_MARKERS = {
    "publication-graph-publish": "data-target",
    "publication-graph-entry": "data-target",
    "publication-graph-link": "data-target",
    "publication-graph-label": "data-label",
    "publication-graph-ref": "data-label",
}


def _read_document(html_dir: Path, source: str) -> html.HtmlElement:
    return html.fromstring((_html_path(html_dir, source)).read_text(encoding="utf-8"))


def _validate_reserved_markers(source: str, document: html.HtmlElement) -> None:
    for marker in document.xpath("//*[starts-with(local-name(), 'publication-graph-')]"):
        marker_name = marker.tag
        required_attr = _RESERVED_MARKERS.get(marker_name)
        if required_attr is None:
            raise ValueError(f"unknown publication-graph marker {marker_name} in {source}")
        if marker.get(required_attr) is None:
            raise ValueError(f"{marker_name} in {source} missing {required_attr}")


def _ownership_edges(source: str, document: html.HtmlElement) -> list[OwnershipEdge]:
    markers = document.xpath("//publication-graph-publish | //publication-graph-entry")
    return [
        OwnershipEdge(source, target)
        for marker in markers
        if (target := marker.get("data-target")) is not None
    ]


def _publication_index_items(source: str, document: html.HtmlElement) -> list[PublicationIndexItem]:
    markers = document.xpath("//publication-graph-publish")
    return [
        PublicationIndexItem(source, target, marker.text_content())
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


def _label_defs(source: str, document: html.HtmlElement) -> list[LabelDef]:
    markers = document.xpath("//publication-graph-label")
    return [
        LabelDef(source, label)
        for marker in markers
        if (label := marker.get("data-label")) is not None
    ]


def _label_refs(source: str, document: html.HtmlElement) -> list[LabelRef]:
    markers = document.xpath("//publication-graph-ref")
    return [
        LabelRef(source, label)
        for marker in markers
        if (label := marker.get("data-label")) is not None
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


def _validated_label_sources(
    rendered: list[str],
    label_defs: list[LabelDef],
    label_refs: list[LabelRef],
) -> dict[str, str]:
    rendered_set = set(rendered)
    label_sources: dict[str, str] = {}

    for label_def in label_defs:
        if label_def.source not in rendered_set:
            continue
        if label_def.label in label_sources:
            raise ValueError(f"duplicate label {label_def.label}")
        label_sources[label_def.label] = label_def.source

    for label_ref in label_refs:
        if label_ref.source in rendered_set and label_ref.label not in label_sources:
            raise ValueError(f"label {label_ref.label} is not rendered")

    return label_sources


def _rewrite_markers(
    document: html.HtmlElement,
    root: str,
    source: str,
    label_sources: dict[str, str],
) -> None:
    for marker in document.xpath("//publication-graph-publish"):
        _remove_element(marker)

    for marker in document.xpath("//publication-graph-link"):
        target = marker.get("data-target")
        if target is None:
            _remove_element(marker)
        else:
            _replace_with_anchor(marker, _route_href(root, target))

    for marker in document.xpath("//publication-graph-entry"):
        _remove_element(marker)

    for marker in document.xpath("//publication-graph-label"):
        label = marker.get("data-label")
        if label is None:
            _remove_element(marker)
        else:
            span = html.Element("span")
            span.set("id", label)
            _replace_element(marker, span)

    for marker in document.xpath("//publication-graph-ref"):
        label = marker.get("data-label")
        if label is None:
            _remove_element(marker)
            continue

        label_source = label_sources[label]
        href = f"#{label}" if label_source == source else f"{_route_href(root, label_source)}#{label}"
        _replace_with_anchor(marker, href)


def _replace_with_anchor(marker: html.HtmlElement, href: str) -> None:
    anchor = html.Element("a")
    anchor.set("href", href)
    _copy_children(marker, anchor)
    _replace_element(marker, anchor)


def _copy_children(source: html.HtmlElement, target: html.HtmlElement) -> None:
    target.text = source.text
    for child in source:
        target.append(deepcopy(child))


def _replace_element(old: html.HtmlElement, new: html.HtmlElement) -> None:
    new.tail = old.tail
    old.getparent().replace(old, new)


def _remove_element(element: html.HtmlElement) -> None:
    parent = element.getparent()
    tail = element.tail
    if tail:
        previous = element.getprevious()
        if previous is None:
            parent.text = (parent.text or "") + tail
        else:
            previous.tail = (previous.tail or "") + tail
    parent.remove(element)


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
    raw = "".join(html.tostring(child, encoding="unicode") for child in body).strip()
    return "\n".join("" if line.strip() == "" else line for line in raw.splitlines())


def _site_page(root: str, title: str, nav_html: str, body: str) -> str:
    if nav_html == "":
        body_html = f"      {body}\n"
    else:
        body_html = f"\n    {body}\n\n"

    return (
        "<!doctype html>\n"
        '<html lang="en">\n'
        "  <head>\n"
        '    <meta charset="utf-8">\n'
        '    <meta name="viewport" content="width=device-width, initial-scale=1">\n'
        f"    <title>{escape(title)} - cair.nz</title>\n"
        "  </head>\n"
        "  <body>\n"
        f'    <header><a href="{_route_href(root, root)}">cair.nz</a></header>\n'
        f"{nav_html}"
        "    <main>\n"
        f"{body_html}"
        "    </main>\n"
        "  </body>\n"
        "</html>\n"
    )


def _navigation_html(
    root: str,
    source: str,
    rendered: list[str],
    ownership_edges: list[OwnershipEdge],
    publication_indices: list[PublicationIndexItem],
) -> str:
    rendered_set = set(rendered)
    index_by_owner: dict[str, list[PublicationIndexItem]] = {}
    for item in publication_indices:
        if item.owner in rendered_set and item.target in rendered_set:
            index_by_owner.setdefault(item.owner, []).append(item)

    return "".join(
        _publication_index_html(root, source, items)
        for owner in _ownership_path(root, source, _rendered_ownership_edges(ownership_edges, rendered_set))
        if (items := index_by_owner.get(owner))
    )


def _rendered_ownership_edges(
    ownership_edges: list[OwnershipEdge],
    rendered_set: set[str],
) -> list[OwnershipEdge]:
    return [
        edge
        for edge in ownership_edges
        if edge.owner in rendered_set and edge.target in rendered_set
    ]


def _ownership_path(root: str, source: str, ownership_edges: list[OwnershipEdge]) -> list[str]:
    owner_by_target = {edge.target: edge.owner for edge in ownership_edges}
    path = [source]
    while path[-1] != root:
        path.append(owner_by_target[path[-1]])
    return list(reversed(path))


def _publication_index_html(root: str, source: str, items: list[PublicationIndexItem]) -> str:
    links = "".join(
        _publication_index_item_html(root, source, item)
        for item in items
    )
    return "    <nav>\n" "      <ul>\n" f"{links}" "      </ul>\n" "    </nav>\n"


def _publication_index_item_html(root: str, source: str, item: PublicationIndexItem) -> str:
    current = ' aria-current="page"' if item.target == source else ""
    return f'        <li><a href="{_route_href(root, item.target)}"{current}>{escape(item.title)}</a></li>\n'


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
