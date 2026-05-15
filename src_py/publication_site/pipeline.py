from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
import os
from pathlib import Path
import shutil

import networkx as nx
from lxml import etree  # ty: ignore[unresolved-import]
from lxml import html

from . import scoping
from .typst_compile import compile_typst_html, compile_typst_pdf


def build_site(
    src_dir: Path,
    html_dir: Path,
    out_dir: Path,
    pdf_typ: Path,
    pdf_out: Path,
    root: str,
    css_href: str | None = None,
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

    transform_site(src_dir, html_dir, out_dir, pdf_typ, root, sources, css_href)
    compile_typst_pdf(pdf_typ, pdf_out, root=_compile_root(src_dir, pdf_typ, pdf_out))


def transform_site(
    src_dir: Path,
    html_dir: Path,
    out_dir: Path,
    pdf_typ: Path,
    root: str,
    sources: list[str],
    css_href: str | None = None,
) -> None:
    documents = {source: _read_document(html_dir, source) for source in sources}
    for source, document in documents.items():
        _validate_reserved_markers(source, document)

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
    stylesheet_hrefs = {
        source: _stylesheet_href(document)
        for source, document in documents.items()
    }
    rendered = _validated_ownership_preorder(root, sources, ownership_edges, link_edges)
    label_sources = _validated_label_sources(rendered, label_defs, label_refs)

    rendered_documents = [(source, documents[source]) for source in rendered]
    context_starts = scoping.discover_context_starts(rendered_documents)
    facts = scoping.discover_facts(
        rendered_documents,
        [scoping.extract_publication_link_facts],
    )
    memberships = scoping.resolve_memberships(
        rendered,
        [(edge.owner, edge.target) for edge in ownership_edges],
        context_starts,
    )
    fact_spaces = scoping.attach_facts(memberships, facts)
    scoping.run_renderers([], fact_spaces)

    for source in rendered:
        document = documents[source]
        _rewrite_markers(document, root, source, label_sources)
        title = _title(document, source)
        out_path = _public_path(out_dir, root, source)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        stylesheet_href = stylesheet_hrefs[source] or css_href
        out_path.write_text(
            _site_page(root, title, _body_html(document), stylesheet_href),
            encoding="utf-8",
        )

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
class LabelDef:
    source: str
    label: str


@dataclass(frozen=True)
class LabelRef:
    source: str
    label: str


_RESERVED_MARKERS: dict[str, str | None] = {
    "publication-graph-publish": "data-target",
    "publication-graph-entry": "data-target",
    "publication-graph-link": "data-target",
    "publication-graph-label": "data-label",
    "publication-graph-ref": "data-label",
    "publication-graph-stylesheet": "data-href",
    "publication-graph-context": None,
}


def _read_document(html_dir: Path, source: str) -> html.HtmlElement:
    return html.fromstring((_html_path(html_dir, source)).read_text(encoding="utf-8"))


def _validate_reserved_markers(source: str, document: html.HtmlElement) -> None:
    for marker in document.xpath("//*[starts-with(local-name(), 'publication-graph-')]"):
        marker_name = marker.tag
        if marker_name not in _RESERVED_MARKERS:
            raise ValueError(f"unknown publication-graph marker {marker_name} in {source}")
        required_attr = _RESERVED_MARKERS[marker_name]
        if required_attr is not None and marker.get(required_attr) is None:
            raise ValueError(f"{marker_name} in {source} missing {required_attr}")


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


def _stylesheet_href(document: html.HtmlElement) -> str | None:
    markers = document.xpath("//publication-graph-stylesheet")
    if not markers:
        return None
    return markers[-1].get("data-href")


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
    _stitch_marker_split_paragraphs(document)

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

    for marker in document.xpath("//publication-graph-stylesheet"):
        _remove_element(marker)

    for marker in document.xpath("//publication-graph-context"):
        _remove_element(marker)


def _stitch_marker_split_paragraphs(document: html.HtmlElement) -> None:
    inline_markers = {"publication-graph-link", "publication-graph-ref", "publication-graph-label"}
    changed = True
    while changed:
        changed = False
        for paragraph in document.xpath("//p"):
            marker = paragraph.getnext()
            if marker is None or marker.tag not in inline_markers:
                continue
            next_paragraph = marker.getnext()
            if next_paragraph is None or next_paragraph.tag != "p":
                continue
            _append_inline(paragraph, marker)
            _append_inline(paragraph, next_paragraph)
            changed = True
            break


def _append_inline(paragraph: html.HtmlElement, element: html.HtmlElement) -> None:
    paragraph.append(deepcopy(element))
    if element.tag == "p":
        _unwrap_last_child(paragraph)
    element.getparent().remove(element)


def _unwrap_last_child(paragraph: html.HtmlElement) -> None:
    child = paragraph[-1]
    if child.text:
        if len(paragraph) > 1:
            previous = paragraph[-2]
            previous.tail = (previous.tail or "") + child.text
        else:
            paragraph.text = (paragraph.text or "") + child.text
    for grandchild in list(child):
        child.remove(grandchild)
        paragraph.append(grandchild)
    if child.tail:
        if len(paragraph) > 1:
            previous = paragraph[-2]
            previous.tail = (previous.tail or "") + child.tail
        else:
            paragraph.text = (paragraph.text or "") + child.tail
    paragraph.remove(child)


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


def _site_page(root: str, title: str, body: str, stylesheet_href: str | None) -> str:
    document = _site_page_document(root, title, body, stylesheet_href)
    page_body = document.find("body")
    head = document.find("head")
    header = page_body.find("header")
    main = page_body.find("main")

    return (
        "<!doctype html>\n"
        '<html lang="en">\n'
        "  <head>\n"
        + "".join(f"    {_element_html(child)}\n" for child in head)
        +
        "  </head>\n"
        "  <body>\n"
        f"    {_element_html(header)}\n"
        "    <main>\n"
        f"{_main_html(main)}"
        "    </main>\n"
        "  </body>\n"
        "</html>\n"
    )


def _site_page_document(
    root: str,
    title: str,
    body: str,
    stylesheet_href: str | None,
) -> html.HtmlElement:
    document = html.Element("html")
    document.set("lang", "en")

    head = etree.SubElement(document, "head")
    etree.SubElement(head, "meta", charset="utf-8")
    etree.SubElement(head, "meta", name="viewport", content="width=device-width, initial-scale=1")
    if stylesheet_href is not None:
        etree.SubElement(head, "link", rel="stylesheet", href=stylesheet_href)
    title_element = etree.SubElement(head, "title")
    title_element.text = f"{title} - cair.nz"

    page_body = etree.SubElement(document, "body")
    header = etree.SubElement(page_body, "header")
    home_link = etree.SubElement(header, "a", href=_route_href(root, root))
    home_link.text = "cair.nz"

    main = etree.SubElement(page_body, "main")
    for child in html.fragments_fromstring(body):
        main.append(child)

    return document


def _element_html(element: html.HtmlElement) -> str:
    return html.tostring(element, encoding="unicode", with_tail=False)


def _main_html(main: html.HtmlElement) -> str:
    body = "".join(html.tostring(child, encoding="unicode", with_tail=True) for child in main)
    return f"      {body}\n"


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
