from __future__ import annotations

from collections.abc import Sequence

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


def test_discover_context_starts_supports_anonymous_marker() -> None:
    document = _document(
        "<publication-graph-context></publication-graph-context>"
    )

    starts = scoping.discover_context_starts([("a.typ", document)])

    assert len(starts) == 1
    assert starts[0].publication == "a.typ"
    assert starts[0].is_anonymous is True


def test_discover_context_starts_treats_empty_name_as_anonymous() -> None:
    document = _document(
        '<publication-graph-context data-name=""></publication-graph-context>'
    )

    starts = scoping.discover_context_starts([("a.typ", document)])

    assert starts[0].is_anonymous is True


def test_discover_context_starts_distinguishes_multiple_anonymous_in_one_publication() -> None:
    document = _document(
        "<publication-graph-context></publication-graph-context>"
        "<publication-graph-context></publication-graph-context>"
    )

    starts = scoping.discover_context_starts([("a.typ", document)])

    assert len(starts) == 2
    assert starts[0].name != starts[1].name
    assert all(start.is_anonymous for start in starts)


def test_anonymous_contexts_in_siblings_isolate_facts() -> None:
    document_b = _document("<publication-graph-context></publication-graph-context>")
    document_c = _document("<publication-graph-context></publication-graph-context>")
    starts = scoping.discover_context_starts([("b.typ", document_b), ("c.typ", document_c)])
    edges = [("a.typ", "b.typ"), ("a.typ", "c.typ")]

    memberships = scoping.resolve_memberships(["a.typ", "b.typ", "c.typ"], edges, starts)
    fact_b = scoping.Fact("publication-link", ("x.typ", "X"), "b.typ", 0, (0,))
    fact_c = scoping.Fact("publication-link", ("y.typ", "Y"), "c.typ", 0, (0,))
    fact_spaces = scoping.attach_facts(memberships, [fact_b, fact_c])

    keys = sorted(fact_spaces.keys())
    assert len(keys) == 2
    by_start = {start: fact_spaces[(name, start)] for name, start in keys}
    assert by_start["b.typ"].facts == (fact_b,)
    assert by_start["c.typ"].facts == (fact_c,)
    assert all(fs.is_anonymous for fs in fact_spaces.values())


def _fact(emitter: str, source_index: int = 0, category: str = "publication-link", value: object = "v") -> scoping.Fact:
    return scoping.Fact(
        category=category,
        value=value,
        emitter=emitter,
        source_index=source_index,
        tree_index=(source_index,),
    )


def _build_fact_spaces(
    *,
    rendered: Sequence[str],
    edges: Sequence[tuple[str, str]],
    starts: Sequence[scoping.ContextStart],
    facts: Sequence[scoping.Fact],
) -> dict[tuple[str, str], scoping.FactSpace]:
    memberships = scoping.resolve_memberships(rendered, edges, starts)
    return scoping.attach_facts(memberships, facts)


def test_predicate_renderer_returns_empty_string_when_no_contexts() -> None:
    renderer = scoping.PredicateRenderer()

    outputs = scoping.run_renderers([renderer], {})

    assert outputs == {"predicate-renderer": ""}


def test_predicate_renderer_handles_single_context_with_no_facts() -> None:
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ"],
        edges=[],
        starts=[scoping.ContextStart(publication="a.typ", name="bib")],
        facts=[],
    )

    output = scoping.run_renderers([scoping.PredicateRenderer()], fact_spaces)["predicate-renderer"]

    assert output == "bib@a.typ:"


def test_predicate_renderer_handles_single_context_with_one_fact() -> None:
    fact = _fact("a.typ", value=("x.typ", "X"))
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ"],
        edges=[],
        starts=[scoping.ContextStart(publication="a.typ", name="bib")],
        facts=[fact],
    )

    output = scoping.run_renderers([scoping.PredicateRenderer()], fact_spaces)["predicate-renderer"]

    assert output == "bib@a.typ:\n  publication-link=('x.typ', 'X') <- a.typ"


def test_predicate_renderer_handles_single_context_with_multiple_facts() -> None:
    fact_a0 = _fact("a.typ", source_index=0, value="A0")
    fact_a1 = _fact("a.typ", source_index=1, value="A1")
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ"],
        edges=[],
        starts=[scoping.ContextStart(publication="a.typ", name="bib")],
        facts=[fact_a1, fact_a0],
    )

    output = scoping.run_renderers([scoping.PredicateRenderer()], fact_spaces)["predicate-renderer"]

    assert output == (
        "bib@a.typ:\n"
        "  publication-link='A0' <- a.typ\n"
        "  publication-link='A1' <- a.typ"
    )


def test_predicate_renderer_reads_multiple_contexts() -> None:
    fact_a = _fact("a.typ", value="A")
    fact_b = _fact("b.typ", value="B")
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ", "b.typ"],
        edges=[("a.typ", "b.typ")],
        starts=[
            scoping.ContextStart(publication="a.typ", name="bib"),
            scoping.ContextStart(publication="b.typ", name="figures"),
        ],
        facts=[fact_a, fact_b],
    )

    output = scoping.run_renderers([scoping.PredicateRenderer()], fact_spaces)["predicate-renderer"]

    assert output == (
        "bib@a.typ:\n"
        "  publication-link='A' <- a.typ\n"
        "  publication-link='B' <- b.typ\n"
        "figures@b.typ:\n"
        "  publication-link='B' <- b.typ"
    )


def test_predicate_renderer_filters_by_context_predicate() -> None:
    fact_a = _fact("a.typ", value="A")
    fact_b = _fact("b.typ", value="B")
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ", "b.typ"],
        edges=[("a.typ", "b.typ")],
        starts=[
            scoping.ContextStart(publication="a.typ", name="bib"),
            scoping.ContextStart(publication="b.typ", name="figures"),
        ],
        facts=[fact_a, fact_b],
    )
    renderer = scoping.PredicateRenderer(
        context_predicate=lambda name: name == "figures"
    )

    output = scoping.run_renderers([renderer], fact_spaces)["predicate-renderer"]

    assert output == "figures@b.typ:\n  publication-link='B' <- b.typ"


def test_predicate_renderer_filters_by_fact_predicate() -> None:
    fact_link = _fact("a.typ", category="publication-link", value="L")
    fact_other = _fact("a.typ", source_index=1, category="other", value="O")
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ"],
        edges=[],
        starts=[scoping.ContextStart(publication="a.typ", name="bib")],
        facts=[fact_link, fact_other],
    )
    renderer = scoping.PredicateRenderer(
        fact_predicate=lambda fact: fact.category == "publication-link"
    )

    output = scoping.run_renderers([renderer], fact_spaces)["predicate-renderer"]

    assert output == "bib@a.typ:\n  publication-link='L' <- a.typ"


def test_predicate_renderer_handles_named_and_anonymous_contexts_together() -> None:
    fact_named = _fact("a.typ", value="N")
    fact_anon = _fact("b.typ", value="A")
    documents = [
        ("a.typ", _document('<publication-graph-context data-name="bib"></publication-graph-context>')),
        ("b.typ", _document("<publication-graph-context></publication-graph-context>")),
    ]
    starts = scoping.discover_context_starts(documents)
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ", "b.typ"],
        edges=[("a.typ", "b.typ")],
        starts=starts,
        facts=[fact_named, fact_anon],
    )

    output = scoping.run_renderers([scoping.PredicateRenderer()], fact_spaces)["predicate-renderer"]

    assert "bib@a.typ:" in output
    assert "anonymous@b.typ:" in output
    assert "publication-link='N' <- a.typ" in output
    assert "publication-link='A' <- b.typ" in output


def test_predicate_renderer_can_select_only_anonymous_contexts() -> None:
    documents = [
        ("a.typ", _document('<publication-graph-context data-name="bib"></publication-graph-context>')),
        ("b.typ", _document("<publication-graph-context></publication-graph-context>")),
    ]
    starts = scoping.discover_context_starts(documents)
    fact_a = _fact("a.typ", value="N")
    fact_b = _fact("b.typ", value="A")
    fact_spaces = _build_fact_spaces(
        rendered=["a.typ", "b.typ"],
        edges=[("a.typ", "b.typ")],
        starts=starts,
        facts=[fact_a, fact_b],
    )
    anonymous_names = {
        name for (name, _), space in fact_spaces.items() if space.is_anonymous
    }
    renderer = scoping.PredicateRenderer(
        id="anon-only",
        context_predicate=lambda name: name in anonymous_names,
    )

    output = scoping.run_renderers([renderer], fact_spaces)["anon-only"]

    assert "bib@a.typ:" not in output
    assert "anonymous@b.typ:" in output


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
