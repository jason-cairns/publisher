:- use_module(library(dcgs)).
:- use_module(library(dif)).
:- use_module(library(lists)).
:- use_module(library(os)).
:- use_module(library(pio)).

:- initialization(main).

main :-
    argv(Args),
    main_args(Args).

main_args([SrcDir, HtmlDir, OutputDir, RootSource|Sources]) :-
    site(SrcDir, HtmlDir, OutputDir, RootSource, Sources),
    halt.
main_args([_, _, _, _|_]) :-
    format("invalid publication ownership graph~n", []),
    halt(1).
main_args([]) :-
    usage.
main_args([_]) :-
    usage.
main_args([_, _]) :-
    usage.
main_args([_, _, _]) :-
    usage.

usage :-
    format("usage: scryer-prolog site.pl -- SRC_DIR HTML_DIR OUTPUT_DIR ROOT_SOURCE SOURCES...~n", []),
    halt(1).

site(SrcDir, HtmlDir, OutputDir, RootSource, Sources) :-
    source_member(RootSource, Sources),
    sources_documents(HtmlDir, Sources, Documents),
    documents_edges(Documents, OwnershipEdges, ReferenceEdges),
    rendered_set(RootSource, Sources, OwnershipEdges, Rendered),
    documents_in(Rendered, Documents, RenderedDocuments),
    own_edges_from(OwnershipEdges, Rendered, RenderedOwnership),
    ref_edges_from(ReferenceEdges, Rendered, RenderedReferences),
    edges_targets_present(RenderedOwnership, Rendered),
    edges_targets_present(RenderedReferences, Rendered),
    root_not_owned(RootSource, RenderedOwnership),
    unique_owned_targets(RenderedOwnership),
    valid_owners(Rendered, RootSource, RenderedOwnership),
    documents_site_files(SrcDir, OutputDir, RenderedDocuments, RenderedOwnership, RootSource).

sources_documents(_, [], []).
sources_documents(HtmlDir, [Source|Sources], [document(Source, Body, Edges)|Documents]) :-
    source_html_file(HtmlDir, Source, HtmlPath),
    phrase_from_file(seq(Html), HtmlPath),
    html_body(Html, Body),
    html_edges(Html, Edges),
    sources_documents(HtmlDir, Sources, Documents).

documents_edges([], [], []).
documents_edges([document(Source, _, Edges)|Documents], OwnershipEdges, ReferenceEdges) :-
    document_edges(Source, Edges, DocumentOwnershipEdges, DocumentReferenceEdges),
    documents_edges(Documents, RestOwnershipEdges, RestReferenceEdges),
    append(DocumentOwnershipEdges, RestOwnershipEdges, OwnershipEdges),
    append(DocumentReferenceEdges, RestReferenceEdges, ReferenceEdges).

document_edges(_, [], [], []).
document_edges(Source, [edge(nav, Target, Label)|Edges], [owns(Source, Target, nav, Label)|OwnershipEdges], ReferenceEdges) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges).
document_edges(Source, [edge(publish, Target, [])|Edges], [owns(Source, Target, publish, [])|OwnershipEdges], ReferenceEdges) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges).
document_edges(Source, [edge(link, Target, Label)|Edges], OwnershipEdges, [refers(Source, Target, Label)|ReferenceEdges]) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges).

edges_targets_present([], _).
edges_targets_present([owns(_, Target, _, _)|Edges], Sources) :-
    source_member(Target, Sources),
    edges_targets_present(Edges, Sources).
edges_targets_present([refers(_, Target, _)|Edges], Sources) :-
    source_member(Target, Sources),
    edges_targets_present(Edges, Sources).

rendered_set(RootSource, Sources, OwnershipEdges, Rendered) :-
    rendered_closure([RootSource], OwnershipEdges, Sources, [RootSource], Rendered).

rendered_closure([], _, _, Rendered, Rendered).
rendered_closure([Source|Queue], OwnershipEdges, Sources, Acc, Rendered) :-
    children_in_sources(Source, OwnershipEdges, Sources, Children),
    add_new_members(Children, Acc, NewMembers, AccUpdated),
    append(Queue, NewMembers, QueueUpdated),
    rendered_closure(QueueUpdated, OwnershipEdges, Sources, AccUpdated, Rendered).

children_in_sources(_, [], _, []).
children_in_sources(Source, [owns(Source, Child, _, _)|Edges], Sources, [Child|Children]) :-
    source_member(Child, Sources),
    children_in_sources(Source, Edges, Sources, Children).
children_in_sources(Source, [owns(Source, Child, _, _)|Edges], Sources, Children) :-
    source_not_member(Child, Sources),
    children_in_sources(Source, Edges, Sources, Children).
children_in_sources(Source, [owns(Other, _, _, _)|Edges], Sources, Children) :-
    dif(Source, Other),
    children_in_sources(Source, Edges, Sources, Children).

add_new_members([], Acc, [], Acc).
add_new_members([X|Xs], Acc, [X|New], FinalAcc) :-
    source_not_member(X, Acc),
    add_new_members(Xs, [X|Acc], New, FinalAcc).
add_new_members([X|Xs], Acc, New, FinalAcc) :-
    source_member(X, Acc),
    add_new_members(Xs, Acc, New, FinalAcc).

documents_in(_, [], []).
documents_in(Rendered, [document(Source, Body, Edges)|Documents], [document(Source, Body, Edges)|Filtered]) :-
    source_member(Source, Rendered),
    documents_in(Rendered, Documents, Filtered).
documents_in(Rendered, [document(Source, _, _)|Documents], Filtered) :-
    source_not_member(Source, Rendered),
    documents_in(Rendered, Documents, Filtered).

own_edges_from([], _, []).
own_edges_from([owns(Owner, Target, Kind, Label)|Edges], Rendered, [owns(Owner, Target, Kind, Label)|Within]) :-
    source_member(Owner, Rendered),
    own_edges_from(Edges, Rendered, Within).
own_edges_from([owns(Owner, _, _, _)|Edges], Rendered, Within) :-
    source_not_member(Owner, Rendered),
    own_edges_from(Edges, Rendered, Within).

ref_edges_from([], _, []).
ref_edges_from([refers(Source, Target, Label)|Edges], Rendered, [refers(Source, Target, Label)|Within]) :-
    source_member(Source, Rendered),
    ref_edges_from(Edges, Rendered, Within).
ref_edges_from([refers(Source, _, _)|Edges], Rendered, Within) :-
    source_not_member(Source, Rendered),
    ref_edges_from(Edges, Rendered, Within).

valid_owners([], _, _).
valid_owners([Source|Sources], RootSource, OwnershipEdges) :-
    valid_owner(Source, RootSource, OwnershipEdges),
    valid_owners(Sources, RootSource, OwnershipEdges).

valid_owner(RootSource, RootSource, OwnershipEdges) :-
    owners(RootSource, OwnershipEdges, []).
valid_owner(Source, RootSource, OwnershipEdges) :-
    dif(Source, RootSource),
    owners(Source, OwnershipEdges, [_]).

owners(_, [], []).
owners(Source, [owns(Owner, Source, _, _)|Edges], [Owner|Owners]) :-
    owners(Source, Edges, Owners).
owners(Source, [owns(_, Target, _, _)|Edges], Owners) :-
    dif(Source, Target),
    owners(Source, Edges, Owners).

root_not_owned(_, []).
root_not_owned(RootSource, [owns(_, Target, _, _)|Edges]) :-
    dif(RootSource, Target),
    root_not_owned(RootSource, Edges).

unique_owned_targets([]).
unique_owned_targets([owns(_, Target, _, _)|Edges]) :-
    target_not_owned_again(Target, Edges),
    unique_owned_targets(Edges).

target_not_owned_again(_, []).
target_not_owned_again(Target, [owns(_, Other, _, _)|Edges]) :-
    dif(Target, Other),
    target_not_owned_again(Target, Edges).

documents_site_files(_, OutputDir, Documents, OwnershipEdges, RootSource) :-
    documents_site_files_(OutputDir, Documents, Documents, OwnershipEdges, RootSource).

documents_site_files_(_, [], _, _, _).
documents_site_files_(OutputDir, [document(Source, Body, _)|Documents], AllDocuments, OwnershipEdges, RootSource) :-
    ownership_path(RootSource, Source, OwnershipEdges, Path),
    nav_bars(RootSource, Path, AllDocuments, NavBars),
    final_body(RootSource, Body, CleanBody),
    site_page(RootSource, Source, NavBars, CleanBody, Page),
    source_public_file(OutputDir, RootSource, Source, OutputPath),
    file_chars(OutputPath, Page),
    documents_site_files_(OutputDir, Documents, AllDocuments, OwnershipEdges, RootSource).

ownership_path(Source, Source, _, [Source]).
ownership_path(RootSource, Source, OwnershipEdges, [RootSource|Path]) :-
    owns_child(RootSource, Child, OwnershipEdges),
    ownership_path(Child, Source, OwnershipEdges, Path).

owns_child(Source, Target, [owns(Source, Target, _, _)|_]).
owns_child(Source, Target, [owns(Other, _, _, _)|Edges]) :-
    dif(Source, Other),
    owns_child(Source, Target, Edges).
owns_child(Source, Target, [owns(Source, Other, _, _)|Edges]) :-
    dif(Target, Other),
    owns_child(Source, Target, Edges).

nav_bars(_, [], _, []).
nav_bars(RootSource, [Source|Sources], Documents, NavBars) :-
    document_nav_entries(Documents, Source, []),
    nav_bars(RootSource, Sources, Documents, NavBars).
nav_bars(RootSource, [Source|Sources], Documents, [NavBar|NavBars]) :-
    document_nav_entries(Documents, Source, [Entry|Entries]),
    phrase(nav_bar(RootSource, [Entry|Entries]), NavBar),
    nav_bars(RootSource, Sources, Documents, NavBars).

document_nav_entries([document(Source, _, Edges)|_], Source, Entries) :-
    nav_entries(Edges, Entries).
document_nav_entries([document(Other, _, _)|Documents], Source, Entries) :-
    dif(Source, Other),
    document_nav_entries(Documents, Source, Entries).

nav_entries([], []).
nav_entries([edge(nav, Target, Label)|Edges], [nav_entry(Target, Label)|Entries]) :-
    nav_entries(Edges, Entries).
nav_entries([edge(publish, _, _)|Edges], Entries) :-
    nav_entries(Edges, Entries).
nav_entries([edge(link, _, _)|Edges], Entries) :-
    nav_entries(Edges, Entries).

nav_bar(RootSource, Entries) -->
    "    <nav>\n",
    "      <ul>\n",
    nav_items(RootSource, Entries),
    "      </ul>\n",
    "    </nav>\n".

nav_items(_, []) --> [].
nav_items(RootSource, [nav_entry(Target, Label)|Entries]) -->
    "        <li><a href=\"",
    route_href(RootSource, Target),
    "\">",
    seq(Label),
    "</a></li>\n",
    nav_items(RootSource, Entries).

final_body(RootSource, Body, CleanBody) :-
    phrase(clean_body(RootSource, CleanBody), Body).

clean_body(_, []) --> [].
clean_body(RootSource, CleanBody) -->
    nav_marker(_Target, _Label),
    clean_body(RootSource, CleanBody).
clean_body(RootSource, CleanBody) -->
    publish_marker(_Target),
    clean_body(RootSource, CleanBody).
clean_body(RootSource, CleanBody) -->
    link_marker(Target, Label),
    { phrase(publication_anchor(RootSource, Target, Label), Anchor) },
    clean_body(RootSource, Rest),
    { append(Anchor, Rest, CleanBody) }.
clean_body(RootSource, [C|Cs]) -->
    [C],
    clean_body(RootSource, Cs).

site_page(RootSource, Source, NavBars, Body, Page) :-
    phrase(site_page_(RootSource, Source, NavBars, Body), Page).

site_page_(RootSource, Source, NavBars, Body) -->
    "<!doctype html>\n",
    "<html lang=\"en\">\n",
    "  <head>\n",
    "    <meta charset=\"utf-8\">\n",
    "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
    "    <title>",
    seq(Source),
    " - cair.nz</title>\n",
    "  </head>\n",
    "  <body>\n",
    "    <header><a href=\"",
    route_href(RootSource, RootSource),
    "\">cair.nz</a></header>\n",
    nav_bar_blocks(NavBars),
    "    <main>\n",
    Body,
    "\n    </main>\n",
    "  </body>\n",
    "</html>\n".

nav_bar_blocks([]) --> [].
nav_bar_blocks([NavBar|NavBars]) -->
    seq(NavBar),
    nav_bar_blocks(NavBars).

publication_anchor(RootSource, Target, Label) -->
    "<a href=\"",
    route_href(RootSource, Target),
    "\">",
    seq(Label),
    "</a>".

html_body(Html, Body) :-
    phrase(html_body_(Body), Html).

html_body_(Body) -->
    any_chars,
    "<body>",
    body_chars(Body),
    "</body>",
    any_chars.

html_edges(Html, Edges) :-
    phrase(html_edges_(Edges), Html).

html_edges_([edge(nav, Target, Label)|Edges]) -->
    nav_marker(Target, Label),
    html_edges_(Edges).
html_edges_([edge(publish, Target, [])|Edges]) -->
    publish_marker(Target),
    html_edges_(Edges).
html_edges_([edge(link, Target, Label)|Edges]) -->
    link_marker(Target, Label),
    html_edges_(Edges).
html_edges_(Edges) -->
    non_marker_char(_),
    html_edges_(Edges).
html_edges_([]) --> [].

nav_marker(Target, Label) -->
    "<cairnz-nav data-target=\"",
    attr_value(Target),
    ">",
    nav_body(Label).

publish_marker(Target) -->
    "<cairnz-publish data-target=\"",
    attr_value(Target),
    "></cairnz-publish>".

link_marker(Target, Label) -->
    "<cairnz-link data-target=\"",
    attr_value(Target),
    ">",
    link_body(Label).

attr_value([]) --> "\"".
attr_value([C|Cs]) --> [C], { dif(C, '"') }, attr_value(Cs).

nav_body([]) --> "</cairnz-nav>".
nav_body([C|Cs]) --> [C], { dif(C, '<') }, nav_body(Cs).

link_body([]) --> "</cairnz-link>".
link_body([C|Cs]) --> [C], { dif(C, '<') }, link_body(Cs).

non_marker_char(C) -->
    [C],
    { dif(C, '<') }.
non_marker_char('<') -->
    "<",
    not_marker_prefix.

% Commit point: at "<cairnz-" the parser MUST match a marker, not
% character-eat the prefix. Without this, html_edges//1 is
% non-deterministic and validation can be bypassed by backtracking into
% an alternative parse where a marker was never extracted.
not_marker_prefix --> not_marker_prefix_after("cairnz-").

not_marker_prefix_after([Expected|_]) -->
    [C],
    { dif(C, Expected) }.
not_marker_prefix_after([Expected|Rest]) -->
    [Expected],
    not_marker_prefix_after(Rest).

any_chars --> [].
any_chars --> [_], any_chars.

body_chars([]) --> [].
body_chars([C|Cs]) --> [C], body_chars(Cs).

source_member(Source, [Source|_]).
source_member(Source, [Other|Sources]) :-
    dif(Source, Other),
    source_member(Source, Sources).

source_not_member(_, []).
source_not_member(Source, [Other|Sources]) :-
    dif(Source, Other),
    source_not_member(Source, Sources).

source_html_file(HtmlDir, Source, Path) :-
    source_stem(Source, Stem),
    phrase(path_file(HtmlDir, Stem, ".html"), Path).

source_public_file(OutputDir, RootSource, RootSource, Path) :-
    phrase(path_file(OutputDir, "index", ".html"), Path).
source_public_file(OutputDir, RootSource, Source, Path) :-
    dif(Source, RootSource),
    source_stem(Source, Stem),
    phrase(path_file(OutputDir, Stem, "/index.html"), Path).

route_href(RootSource, RootSource) --> "/".
route_href(RootSource, Source) -->
    { dif(Source, RootSource), source_stem(Source, Stem) },
    "/",
    seq(Stem),
    "/".

source_stem(Source, Stem) :-
    phrase(source_stem_(Stem), Source).

source_stem_(Stem) -->
    seq(Stem),
    ".typ".

path_file(Directory, Stem, Extension) -->
    seq(Directory),
    "/",
    seq(Stem),
    seq(Extension).

file_chars(Path, Chars) :-
    open(Path, write, Stream),
    stream_chars(Stream, Chars),
    close(Stream).

stream_chars(_, []).
stream_chars(Stream, [C|Cs]) :-
    put_char(Stream, C),
    stream_chars(Stream, Cs).
