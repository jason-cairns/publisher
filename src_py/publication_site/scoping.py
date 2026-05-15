"""General publication fact, context, and rendering model.

ADR 0007 defines the conceptual model; this module implements the data types
and the two attribute-grammar-shaped passes over the publication ownership
tree:

- inherited attributes  → context membership (down-pass)
- synthesized attributes → facts attached to scoped fact spaces (up-pass)

JAS-44 ships the machinery and a single fact extractor that turns
``publication-graph-publish`` markers into ``publication-link`` facts.
Concrete renderer instances and additional fact extractors land with the
tickets that consume them.
"""

from __future__ import annotations

from collections.abc import Callable, Iterable, Sequence
from dataclasses import dataclass
from typing import Protocol

from lxml import html


@dataclass(frozen=True, slots=True)
class Fact:
    category: str
    value: object
    emitter: str
    source_index: int
    tree_index: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class ContextStart:
    publication: str
    name: str


@dataclass(frozen=True, slots=True)
class ContextMembership:
    publication: str
    name: str
    start_publication: str


@dataclass(frozen=True, slots=True)
class FactSpace:
    name: str
    start_publication: str
    facts: tuple[Fact, ...]


class Renderer(Protocol):
    id: str
    read_contexts: tuple[str, ...]

    def accepts(self, fact: Fact) -> bool: ...

    def render(self, fact_spaces: Sequence[FactSpace]) -> str: ...


FactExtractor = Callable[[str, html.HtmlElement], list[Fact]]


def discover_context_starts(
    documents: Iterable[tuple[str, html.HtmlElement]],
) -> list[ContextStart]:
    starts: list[ContextStart] = []
    for source, document in documents:
        for marker in document.xpath("//publication-graph-context"):
            name = marker.get("data-name")
            if name is None:
                continue
            starts.append(ContextStart(publication=source, name=name))
    return starts


def discover_facts(
    documents: Iterable[tuple[str, html.HtmlElement]],
    extractors: Sequence[FactExtractor],
) -> list[Fact]:
    facts: list[Fact] = []
    for source, document in documents:
        for extractor in extractors:
            facts.extend(extractor(source, document))
    return facts


def extract_publication_link_facts(
    source: str, document: html.HtmlElement
) -> list[Fact]:
    facts: list[Fact] = []
    for index, marker in enumerate(document.xpath("//publication-graph-publish")):
        target = marker.get("data-target")
        if target is None:
            continue
        facts.append(
            Fact(
                category="publication-link",
                value=(target, marker.text_content()),
                emitter=source,
                source_index=index,
                tree_index=_tree_index(marker),
            )
        )
    return facts


def _tree_index(element: html.HtmlElement) -> tuple[int, ...]:
    indices: list[int] = []
    current = element
    parent = current.getparent()
    while parent is not None:
        indices.append(parent.index(current))
        current = parent
        parent = current.getparent()
    return tuple(reversed(indices))


def resolve_memberships(
    rendered: Sequence[str],
    ownership_edges: Iterable[tuple[str, str]],
    context_starts: Sequence[ContextStart],
) -> list[ContextMembership]:
    rendered_set = set(rendered)
    children_by_owner: dict[str, list[str]] = {source: [] for source in rendered}
    for owner, target in ownership_edges:
        if owner in rendered_set and target in rendered_set:
            children_by_owner.setdefault(owner, []).append(target)

    memberships: list[ContextMembership] = []
    for start in context_starts:
        if start.publication not in rendered_set:
            continue
        for descendant in _descendants_inclusive(start.publication, children_by_owner):
            memberships.append(
                ContextMembership(
                    publication=descendant,
                    name=start.name,
                    start_publication=start.publication,
                )
            )
    return memberships


def _descendants_inclusive(
    root: str, children_by_owner: dict[str, list[str]]
) -> list[str]:
    visited: list[str] = []
    seen: set[str] = set()
    stack = [root]
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        visited.append(current)
        stack.extend(reversed(children_by_owner.get(current, [])))
    return visited


def attach_facts(
    memberships: Sequence[ContextMembership],
    facts: Sequence[Fact],
) -> dict[tuple[str, str], FactSpace]:
    members_by_context: dict[tuple[str, str], set[str]] = {}
    for membership in memberships:
        key = (membership.name, membership.start_publication)
        members_by_context.setdefault(key, set()).add(membership.publication)

    fact_spaces: dict[tuple[str, str], FactSpace] = {}
    for (name, start_publication), members in members_by_context.items():
        scoped = tuple(
            sorted(
                (fact for fact in facts if fact.emitter in members),
                key=lambda fact: (fact.emitter, fact.source_index),
            )
        )
        fact_spaces[(name, start_publication)] = FactSpace(
            name=name,
            start_publication=start_publication,
            facts=scoped,
        )
    return fact_spaces


def run_renderers(
    renderers: Sequence[Renderer],
    fact_spaces: dict[tuple[str, str], FactSpace],
) -> dict[str, str]:
    outputs: dict[str, str] = {}
    for renderer in renderers:
        selected = tuple(
            FactSpace(
                name=fact_space.name,
                start_publication=fact_space.start_publication,
                facts=tuple(fact for fact in fact_space.facts if renderer.accepts(fact)),
            )
            for (name, _), fact_space in fact_spaces.items()
            if name in renderer.read_contexts
        )
        outputs[renderer.id] = renderer.render(selected)
    return outputs
