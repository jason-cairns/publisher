from __future__ import annotations

from lxml import html

from publication_site import scoping


def _document(body: str) -> html.HtmlElement:
    return html.fromstring(f"<html><body>{body}</body></html>")


def test_discover_context_starts_finds_named_context() -> None:
    document = _document('<publication-graph-context data-name="bib"></publication-graph-context>')

    starts = scoping.discover_context_starts([("a.typ", document)])

    assert starts == [scoping.ContextStart(publication="a.typ", name="bib")]


def test_discover_context_starts_returns_empty_without_markers() -> None:
    starts = scoping.discover_context_starts([("a.typ", _document("<p>nothing</p>"))])

    assert starts == []


def test_discover_context_starts_handles_multiple_documents() -> None:
    a = _document('<publication-graph-context data-name="bib"></publication-graph-context>')
    b = _document('<publication-graph-context data-name="figures"></publication-graph-context>')

    starts = scoping.discover_context_starts([("a.typ", a), ("b.typ", b)])

    assert scoping.ContextStart(publication="a.typ", name="bib") in starts
    assert scoping.ContextStart(publication="b.typ", name="figures") in starts


def test_resolve_memberships_inherits_to_descendants() -> None:
    starts = [scoping.ContextStart(publication="a.typ", name="bib")]
    edges = [("a.typ", "b.typ"), ("b.typ", "c.typ")]

    memberships = scoping.resolve_memberships(["a.typ", "b.typ", "c.typ"], edges, starts)

    publications = {m.publication for m in memberships}
    assert publications == {"a.typ", "b.typ", "c.typ"}
    assert all(m.name == "bib" and m.start_publication == "a.typ" for m in memberships)


def test_resolve_memberships_isolates_siblings() -> None:
    starts = [scoping.ContextStart(publication="b.typ", name="bib")]
    edges = [("a.typ", "b.typ"), ("a.typ", "c.typ")]

    memberships = scoping.resolve_memberships(["a.typ", "b.typ", "c.typ"], edges, starts)

    publications = {m.publication for m in memberships}
    assert publications == {"b.typ"}


def test_resolve_memberships_keeps_nested_same_name_distinct() -> None:
    starts = [
        scoping.ContextStart(publication="a.typ", name="bib"),
        scoping.ContextStart(publication="b.typ", name="bib"),
    ]
    edges = [("a.typ", "b.typ"), ("b.typ", "c.typ")]

    memberships = scoping.resolve_memberships(["a.typ", "b.typ", "c.typ"], edges, starts)

    by_descendant: dict[str, set[str]] = {}
    for membership in memberships:
        by_descendant.setdefault(membership.publication, set()).add(membership.start_publication)

    assert by_descendant["a.typ"] == {"a.typ"}
    assert by_descendant["b.typ"] == {"a.typ", "b.typ"}
    assert by_descendant["c.typ"] == {"a.typ", "b.typ"}


def test_resolve_memberships_skips_unrendered_starts() -> None:
    starts = [scoping.ContextStart(publication="orphan.typ", name="bib")]

    memberships = scoping.resolve_memberships(["a.typ"], [], starts)

    assert memberships == []


def test_extract_publication_link_facts_from_publish() -> None:
    document = _document(
        '<publication-graph-publish data-target="x.typ">Title</publication-graph-publish>'
    )

    facts = scoping.extract_publication_link_facts("a.typ", document)

    assert len(facts) == 1
    assert facts[0].category == "publication-link"
    assert facts[0].value == ("x.typ", "Title")
    assert facts[0].emitter == "a.typ"
    assert facts[0].source_index == 0


def test_extract_publication_link_facts_ignores_entry() -> None:
    document = _document(
        '<publication-graph-entry data-target="x.typ"></publication-graph-entry>'
    )

    facts = scoping.extract_publication_link_facts("a.typ", document)

    assert facts == []


def test_extract_publication_link_facts_indexes_in_source_order() -> None:
    document = _document(
        '<publication-graph-publish data-target="x.typ">First</publication-graph-publish>'
        '<publication-graph-publish data-target="y.typ">Second</publication-graph-publish>'
    )

    facts = scoping.extract_publication_link_facts("a.typ", document)

    assert [f.source_index for f in facts] == [0, 1]
    assert [f.value for f in facts] == [("x.typ", "First"), ("y.typ", "Second")]


def test_attach_facts_scopes_facts_to_member_emitters() -> None:
    fact_a = scoping.Fact(
        category="publication-link",
        value=("x.typ", "X"),
        emitter="a.typ",
        source_index=0,
        tree_index=(0,),
    )
    fact_c = scoping.Fact(
        category="publication-link",
        value=("y.typ", "Y"),
        emitter="c.typ",
        source_index=0,
        tree_index=(0,),
    )
    memberships = [
        scoping.ContextMembership(publication="a.typ", name="bib", start_publication="a.typ"),
        scoping.ContextMembership(publication="b.typ", name="bib", start_publication="a.typ"),
    ]

    fact_spaces = scoping.attach_facts(memberships, [fact_a, fact_c])

    assert ("bib", "a.typ") in fact_spaces
    assert fact_spaces[("bib", "a.typ")].facts == (fact_a,)


def test_attach_facts_isolates_sibling_contexts() -> None:
    fact_b = scoping.Fact(
        category="publication-link",
        value=("x.typ", "X"),
        emitter="b.typ",
        source_index=0,
        tree_index=(0,),
    )
    fact_c = scoping.Fact(
        category="publication-link",
        value=("y.typ", "Y"),
        emitter="c.typ",
        source_index=0,
        tree_index=(0,),
    )
    memberships = [
        scoping.ContextMembership(publication="b.typ", name="bib", start_publication="b.typ"),
        scoping.ContextMembership(publication="c.typ", name="bib", start_publication="c.typ"),
    ]

    fact_spaces = scoping.attach_facts(memberships, [fact_b, fact_c])

    assert fact_spaces[("bib", "b.typ")].facts == (fact_b,)
    assert fact_spaces[("bib", "c.typ")].facts == (fact_c,)


def test_attach_facts_orders_facts_by_emitter_then_source_index() -> None:
    fact_a0 = scoping.Fact("publication-link", ("x.typ", "X"), "a.typ", 0, (0,))
    fact_a1 = scoping.Fact("publication-link", ("y.typ", "Y"), "a.typ", 1, (1,))
    fact_b = scoping.Fact("publication-link", ("z.typ", "Z"), "b.typ", 0, (0,))
    memberships = [
        scoping.ContextMembership(publication="a.typ", name="bib", start_publication="a.typ"),
        scoping.ContextMembership(publication="b.typ", name="bib", start_publication="a.typ"),
    ]

    fact_spaces = scoping.attach_facts(memberships, [fact_b, fact_a1, fact_a0])

    assert fact_spaces[("bib", "a.typ")].facts == (fact_a0, fact_a1, fact_b)


def test_run_renderers_returns_empty_when_no_renderers() -> None:
    assert scoping.run_renderers([], {}) == {}


def test_discover_facts_runs_each_extractor() -> None:
    document = _document(
        '<publication-graph-publish data-target="x.typ">Title</publication-graph-publish>'
    )

    facts = scoping.discover_facts(
        [("a.typ", document)],
        [scoping.extract_publication_link_facts],
    )

    assert len(facts) == 1
    assert facts[0].emitter == "a.typ"
