:- use_module(library(dcgs)).
:- use_module(library(dif)).
:- use_module(library(lists)).
:- use_module(library(os)).
:- use_module(library(pio)).

:- initialization(main).

main :-
    argv(Args),
    main_args(Args).

main_args([SrcDir, HtmlDir, OutputDir, PdfTypPath, RootSource|Sources]) :-
    site(SrcDir, HtmlDir, OutputDir, PdfTypPath, RootSource, Sources),
    halt.
main_args([_, _, _, _, _|_]) :-
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
main_args([_, _, _, _]) :-
    usage.

usage :-
    format("usage: scryer-prolog site.pl -- SRC_DIR HTML_DIR OUTPUT_DIR PDF_TYP_PATH ROOT_SOURCE SOURCES...~n", []),
    halt(1).

site(SrcDir, HtmlDir, OutputDir, PdfTypPath, RootSource, Sources) :-
    source_member(RootSource, Sources),
    sources_documents(HtmlDir, Sources, Documents),
    documents_edges(Documents, OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs),
    rendered_set(RootSource, Sources, OwnershipEdges, Rendered),
    documents_in(Rendered, Documents, RenderedDocuments),
    own_edges_from(OwnershipEdges, Rendered, RenderedOwnership),
    ref_edges_from(ReferenceEdges, Rendered, RenderedReferences),
    label_defs_from(LabelDefs, Rendered, RenderedLabelDefs),
    label_refs_from(LabelRefs, Rendered, RenderedLabelRefs),
    edges_targets_present(RenderedOwnership, Rendered),
    edges_targets_present(RenderedReferences, Rendered),
    unique_label_defs(RenderedLabelDefs),
    label_refs_present(RenderedLabelRefs, RenderedLabelDefs),
    root_not_owned(RootSource, RenderedOwnership),
    unique_owned_targets(RenderedOwnership),
    valid_owners(Rendered, RootSource, RenderedOwnership),
    ownership_preorder(RootSource, RenderedOwnership, PdfOrder),
    documents_site_files(SrcDir, OutputDir, RenderedDocuments, RenderedOwnership, RenderedLabelDefs, RootSource),
    pdf_typ_file(PdfTypPath, SrcDir, PdfOrder, PdfTyp),
    file_chars(PdfTypPath, PdfTyp).

sources_documents(_, [], []).
sources_documents(HtmlDir, [Source|Sources], [document(Source, Body, Edges)|Documents]) :-
    source_html_file(HtmlDir, Source, HtmlPath),
    phrase_from_file(seq(Html), HtmlPath),
    html_body(Html, Body),
    html_edges(Html, Edges),
    sources_documents(HtmlDir, Sources, Documents).

documents_edges([], [], [], [], []).
documents_edges([document(Source, _, Edges)|Documents], OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs) :-
    document_edges(Source, Edges, DocumentOwnershipEdges, DocumentReferenceEdges, DocumentLabelDefs, DocumentLabelRefs),
    documents_edges(Documents, RestOwnershipEdges, RestReferenceEdges, RestLabelDefs, RestLabelRefs),
    append(DocumentOwnershipEdges, RestOwnershipEdges, OwnershipEdges),
    append(DocumentReferenceEdges, RestReferenceEdges, ReferenceEdges),
    append(DocumentLabelDefs, RestLabelDefs, LabelDefs),
    append(DocumentLabelRefs, RestLabelRefs, LabelRefs).

document_edges(_, [], [], [], [], []).
document_edges(Source, [edge(publish, Target, Label)|Edges], [owns(Source, Target, publish, Label)|OwnershipEdges], ReferenceEdges, LabelDefs, LabelRefs) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs).
document_edges(Source, [edge(entry, Target, [])|Edges], [owns(Source, Target, entry, [])|OwnershipEdges], ReferenceEdges, LabelDefs, LabelRefs) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs).
document_edges(Source, [edge(link, Target, Label)|Edges], OwnershipEdges, [refers(Source, Target, Label)|ReferenceEdges], LabelDefs, LabelRefs) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs).
document_edges(Source, [edge(label, Name)|Edges], OwnershipEdges, ReferenceEdges, [defines_label(Source, Name)|LabelDefs], LabelRefs) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs).
document_edges(Source, [edge(label_ref, Name, Label)|Edges], OwnershipEdges, ReferenceEdges, LabelDefs, [refers_label(Source, Name, Label)|LabelRefs]) :-
    document_edges(Source, Edges, OwnershipEdges, ReferenceEdges, LabelDefs, LabelRefs).

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

label_defs_from([], _, []).
label_defs_from([defines_label(Source, Name)|Labels], Rendered, [defines_label(Source, Name)|Within]) :-
    source_member(Source, Rendered),
    label_defs_from(Labels, Rendered, Within).
label_defs_from([defines_label(Source, _)|Labels], Rendered, Within) :-
    source_not_member(Source, Rendered),
    label_defs_from(Labels, Rendered, Within).

label_refs_from([], _, []).
label_refs_from([refers_label(Source, Name, Label)|Labels], Rendered, [refers_label(Source, Name, Label)|Within]) :-
    source_member(Source, Rendered),
    label_refs_from(Labels, Rendered, Within).
label_refs_from([refers_label(Source, _, _)|Labels], Rendered, Within) :-
    source_not_member(Source, Rendered),
    label_refs_from(Labels, Rendered, Within).

unique_label_defs([]).
unique_label_defs([defines_label(_, Name)|Labels]) :-
    label_not_defined_again(Name, Labels),
    unique_label_defs(Labels).

label_not_defined_again(_, []).
label_not_defined_again(Name, [defines_label(_, Other)|Labels]) :-
    dif(Name, Other),
    label_not_defined_again(Name, Labels).

label_refs_present([], _).
label_refs_present([refers_label(_, Name, _)|Refs], LabelDefs) :-
    label_defined(Name, LabelDefs),
    label_refs_present(Refs, LabelDefs).

label_defined(Name, [defines_label(_, Name)|_]).
label_defined(Name, [defines_label(_, Other)|Labels]) :-
    dif(Name, Other),
    label_defined(Name, Labels).

label_source(Name, [defines_label(Source, Name)|_], Source).
label_source(Name, [defines_label(_, Other)|Labels], Source) :-
    dif(Name, Other),
    label_source(Name, Labels, Source).

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

ownership_preorder(RootSource, OwnershipEdges, Order) :-
    preorder_source(RootSource, OwnershipEdges, Order).

preorder_source(Source, OwnershipEdges, [Source|Descendants]) :-
    ownership_children(Source, OwnershipEdges, Children),
    preorder_sources(Children, OwnershipEdges, Descendants).

preorder_sources([], _, []).
preorder_sources([Source|Sources], OwnershipEdges, Order) :-
    preorder_source(Source, OwnershipEdges, SourceOrder),
    preorder_sources(Sources, OwnershipEdges, RestOrder),
    append(SourceOrder, RestOrder, Order).

ownership_children(_, [], []).
ownership_children(Source, [owns(Source, Target, _, _)|Edges], [Target|Children]) :-
    ownership_children(Source, Edges, Children).
ownership_children(Source, [owns(Other, _, _, _)|Edges], Children) :-
    dif(Source, Other),
    ownership_children(Source, Edges, Children).

pdf_typ_file(PdfTypPath, SrcDir, Sources, Typ) :-
    typst_source_prefix(PdfTypPath, SrcDir, SourcePrefix),
    phrase(pdf_typ_file_(SourcePrefix, Sources), Typ).

pdf_typ_file_(SourcePrefix, Sources) -->
    "#import \"",
    seq(SourcePrefix),
    "/_publication.typ\": publication\n\n",
    pdf_publications(SourcePrefix, Sources).

pdf_publications(_, []) --> [].
pdf_publications(SourcePrefix, [Source|Sources]) -->
    "#publication(\"",
    seq(Source),
    "\")[\n",
    "  #include \"",
    seq(SourcePrefix),
    "/",
    seq(Source),
    "\"\n",
    "]\n\n",
    pdf_publications(SourcePrefix, Sources).

typst_source_prefix(PdfTypPath, SrcDir, SourcePrefix) :-
    getenv("PWD", Cwd),
    absolute_path_components(Cwd, PdfTypPath, PdfTypComponents),
    absolute_path_components(Cwd, SrcDir, SrcComponents),
    directory_components(PdfTypComponents, PdfTypDirComponents),
    relative_path_components(PdfTypDirComponents, SrcComponents, SourceComponents),
    path_from_components(SourceComponents, SourcePrefix).

absolute_path_components(_, ['/'|Path], Components) :-
    path_components(Path, RawComponents),
    normalized_path_components(RawComponents, Components).
absolute_path_components(Cwd, [C|Path], Components) :-
    dif(C, '/'),
    path_components(Cwd, CwdComponents),
    path_components([C|Path], PathComponents),
    append(CwdComponents, PathComponents, RawComponents),
    normalized_path_components(RawComponents, Components).

directory_components([_], []).
directory_components([Component, Next|Components], [Component|Directory]) :-
    directory_components([Next|Components], Directory).

relative_path_components(From, To, Relative) :-
    uncommon_path_components(From, To, FromRest, ToRest),
    parent_path_components(FromRest, Parents),
    append(Parents, ToRest, Relative).

uncommon_path_components([], To, [], To).
uncommon_path_components(From, [], From, []).
uncommon_path_components([Component|From], [Component|To], FromRest, ToRest) :-
    uncommon_path_components(From, To, FromRest, ToRest).
uncommon_path_components([FromComponent|From], [ToComponent|To], [FromComponent|From], [ToComponent|To]) :-
    dif(FromComponent, ToComponent).

parent_path_components([], []).
parent_path_components([_|Components], [['.','.']|Parents]) :-
    parent_path_components(Components, Parents).

normalized_path_components(Components, Normalized) :-
    normalized_path_components_(Components, [], Reversed),
    reverse(Reversed, Normalized).

normalized_path_components_([], Components, Components).
normalized_path_components_([[]|Components], Acc, Normalized) :-
    normalized_path_components_(Components, Acc, Normalized).
normalized_path_components_([['.']|Components], Acc, Normalized) :-
    normalized_path_components_(Components, Acc, Normalized).
normalized_path_components_([['.','.']|Components], [_|Acc], Normalized) :-
    normalized_path_components_(Components, Acc, Normalized).
normalized_path_components_([['.','.']|Components], [], Normalized) :-
    normalized_path_components_(Components, [], Normalized).
normalized_path_components_([Component|Components], Acc, Normalized) :-
    dif(Component, []),
    dif(Component, ['.']),
    dif(Component, ['.','.']),
    normalized_path_components_(Components, [Component|Acc], Normalized).

path_components(Path, Components) :-
    phrase(path_components_(Components), Path).

path_components_([Component|Components]) -->
    path_component(Component),
    "/",
    path_components_(Components).
path_components_([Component]) -->
    path_component(Component).
path_components_([]) --> [].

path_component([C|Cs]) -->
    [C],
    { dif(C, '/') },
    path_component(Cs).
path_component([]) --> [].

path_from_components([], ".").
path_from_components(Components, Path) :-
    phrase(path_from_components_(Components), Path).

path_from_components_([Component]) -->
    seq(Component).
path_from_components_([Component, Next|Components]) -->
    seq(Component),
    "/",
    path_from_components_([Next|Components]).

documents_site_files(_, OutputDir, Documents, OwnershipEdges, LabelDefs, RootSource) :-
    documents_site_files_(OutputDir, Documents, Documents, OwnershipEdges, LabelDefs, RootSource).

documents_site_files_(_, [], _, _, _, _).
documents_site_files_(OutputDir, [document(Source, Body, _)|Documents], AllDocuments, OwnershipEdges, LabelDefs, RootSource) :-
    ownership_path(RootSource, Source, OwnershipEdges, Path),
    index_bars(RootSource, Source, Path, AllDocuments, IndexBars),
    final_body(RootSource, Source, LabelDefs, Body, CleanBody),
    blank_line_space_normalized(CleanBody, NormalizedBody),
    document_title(Source, Body, Title),
    site_page(RootSource, Title, IndexBars, NormalizedBody, Page),
    source_public_file(OutputDir, RootSource, Source, OutputPath),
    file_chars(OutputPath, Page),
    documents_site_files_(OutputDir, Documents, AllDocuments, OwnershipEdges, LabelDefs, RootSource).

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

index_bars(_, _, [], _, []).
index_bars(RootSource, CurrentSource, [Source|Sources], Documents, IndexBars) :-
    document_index_entries(Documents, Source, []),
    index_bars(RootSource, CurrentSource, Sources, Documents, IndexBars).
index_bars(RootSource, CurrentSource, [Source|Sources], Documents, [IndexBar|IndexBars]) :-
    document_index_entries(Documents, Source, [Entry|Entries]),
    phrase(index_bar(RootSource, CurrentSource, [Entry|Entries]), IndexBar),
    index_bars(RootSource, CurrentSource, Sources, Documents, IndexBars).

document_index_entries([document(Source, _, Edges)|_], Source, Entries) :-
    index_entries(Edges, Entries).
document_index_entries([document(Other, _, _)|Documents], Source, Entries) :-
    dif(Source, Other),
    document_index_entries(Documents, Source, Entries).

index_entries([], []).
index_entries([edge(publish, Target, Label)|Edges], [index_entry(Target, Label)|Entries]) :-
    index_entries(Edges, Entries).
index_entries([edge(entry, _, _)|Edges], Entries) :-
    index_entries(Edges, Entries).
index_entries([edge(link, _, _)|Edges], Entries) :-
    index_entries(Edges, Entries).
index_entries([edge(label, _)|Edges], Entries) :-
    index_entries(Edges, Entries).
index_entries([edge(label_ref, _, _)|Edges], Entries) :-
    index_entries(Edges, Entries).

index_bar(RootSource, CurrentSource, Entries) -->
    "    <nav>\n",
    "      <ul>\n",
    index_items(RootSource, CurrentSource, Entries),
    "      </ul>\n",
    "    </nav>\n".

index_items(_, _, []) --> [].
index_items(RootSource, CurrentSource, [index_entry(CurrentSource, Label)|Entries]) -->
    "        <li><a href=\"",
    route_href(RootSource, CurrentSource),
    "\" aria-current=\"page\">",
    seq(Label),
    "</a></li>\n",
    index_items(RootSource, CurrentSource, Entries).
index_items(RootSource, CurrentSource, [index_entry(Target, Label)|Entries]) -->
    { dif(CurrentSource, Target) },
    "        <li><a href=\"",
    route_href(RootSource, Target),
    "\">",
    seq(Label),
    "</a></li>\n",
    index_items(RootSource, CurrentSource, Entries).

final_body(RootSource, Source, LabelDefs, Body, CleanBody) :-
    phrase(clean_body(RootSource, Source, LabelDefs, CleanBody), Body).

clean_body(_, _, _, []) --> [].
clean_body(RootSource, Source, LabelDefs, CleanBody) -->
    publish_marker(_Target, _Label),
    clean_body(RootSource, Source, LabelDefs, CleanBody).
clean_body(RootSource, Source, LabelDefs, CleanBody) -->
    entry_marker(_Target),
    clean_body(RootSource, Source, LabelDefs, CleanBody).
clean_body(RootSource, Source, LabelDefs, CleanBody) -->
    link_marker(Target, Label),
    { phrase(publication_anchor(RootSource, Target, Label), Anchor) },
    clean_body(RootSource, Source, LabelDefs, Rest),
    { append(Anchor, Rest, CleanBody) }.
clean_body(RootSource, Source, LabelDefs, CleanBody) -->
    label_marker(Name),
    { phrase(label_anchor(Name), Anchor) },
    clean_body(RootSource, Source, LabelDefs, Rest),
    { append(Anchor, Rest, CleanBody) }.
clean_body(RootSource, Source, LabelDefs, CleanBody) -->
    label_ref_marker(Name, Label),
    { label_source(Name, LabelDefs, TargetSource),
      phrase(label_ref_anchor(RootSource, Source, TargetSource, Name, Label), Anchor) },
    clean_body(RootSource, Source, LabelDefs, Rest),
    { append(Anchor, Rest, CleanBody) }.
clean_body(RootSource, Source, LabelDefs, [C|Cs]) -->
    [C],
    clean_body(RootSource, Source, LabelDefs, Cs).

blank_line_space_normalized(Body, Normalized) :-
    phrase(blank_line_space_normalized_(Normalized), Body).

blank_line_space_normalized_(Normalized) -->
    line_start(Normalized).

line_start([]) --> [].
line_start(Line) -->
    line_spaces(Spaces),
    line_after_spaces(Spaces, Line).

line_spaces([' '|Spaces]) -->
    " ",
    line_spaces(Spaces).
line_spaces([]) --> [].

line_after_spaces(_, ['\n'|Lines]) -->
    "\n",
    line_start(Lines).
line_after_spaces(_, []) --> [].
line_after_spaces(Spaces, Line) -->
    [C],
    { dif(C, '\n') },
    line_tail(Tail),
    { append(Spaces, [C|Tail], Line) }.

line_tail(['\n'|Lines]) -->
    "\n",
    line_start(Lines).
line_tail([C|Tail]) -->
    [C],
    { dif(C, '\n') },
    line_tail(Tail).
line_tail([]) --> [].

document_title(_, Body, Title) :-
    phrase(first_heading(Title), Body).
document_title(Source, Body, Source) :-
    phrase(no_heading, Body).

first_heading(Title) -->
    heading_open(Level),
    heading_text(Level, Title),
    any_chars.
first_heading(Title) -->
    non_heading_char,
    first_heading(Title).

heading_open(Level) -->
    "<h",
    heading_level(Level),
    ">".

heading_close(Level) -->
    "</h",
    [Level],
    ">".

heading_level('1') --> "1".
heading_level('2') --> "2".
heading_level('3') --> "3".
heading_level('4') --> "4".
heading_level('5') --> "5".
heading_level('6') --> "6".

heading_text(Level, []) -->
    heading_close(Level).
heading_text(Level, Text) -->
    inline_html_tag,
    heading_text(Level, Text).
heading_text(Level, [C|Cs]) -->
    [C],
    { dif(C, '<') },
    heading_text(Level, Cs).

inline_html_tag -->
    "<",
    tag_chars.

tag_chars -->
    ">".
tag_chars -->
    [C],
    { dif(C, '>') },
    tag_chars.

no_heading --> [].
no_heading -->
    non_heading_char,
    no_heading.

non_heading_char -->
    [C],
    { dif(C, '<') }.
non_heading_char -->
    "<",
    not_heading_start.

not_heading_start -->
    [C],
    { dif(C, 'h') }.
not_heading_start -->
    "h",
    [C],
    { dif(C, '1'), dif(C, '2'), dif(C, '3'), dif(C, '4'), dif(C, '5'), dif(C, '6') }.
not_heading_start -->
    "h",
    heading_level(_),
    [C],
    { dif(C, '>') }.

site_page(RootSource, Title, IndexBars, Body, Page) :-
    phrase(site_page_(RootSource, Title, IndexBars, Body), Page).

site_page_(RootSource, Title, IndexBars, Body) -->
    "<!doctype html>\n",
    "<html lang=\"en\">\n",
    "  <head>\n",
    "    <meta charset=\"utf-8\">\n",
    "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
    "    <title>",
    seq(Title),
    " - cair.nz</title>\n",
    "  </head>\n",
    "  <body>\n",
    "    <header><a href=\"",
    route_href(RootSource, RootSource),
    "\">cair.nz</a></header>\n",
    index_bar_blocks(IndexBars),
    "    <main>\n",
    Body,
    "\n    </main>\n",
    "  </body>\n",
    "</html>\n".

index_bar_blocks([]) --> [].
index_bar_blocks([IndexBar|IndexBars]) -->
    seq(IndexBar),
    index_bar_blocks(IndexBars).

publication_anchor(RootSource, Target, Label) -->
    "<a href=\"",
    route_href(RootSource, Target),
    "\">",
    seq(Label),
    "</a>".

label_anchor(Name) -->
    "<span id=\"",
    seq(Name),
    "\"></span>".

label_ref_anchor(RootSource, Source, TargetSource, Name, Label) -->
    "<a href=\"",
    label_href(RootSource, Source, TargetSource, Name),
    "\">",
    seq(Label),
    "</a>".

label_href(_, Source, Source, Name) -->
    "#",
    seq(Name).
label_href(RootSource, Source, TargetSource, Name) -->
    { dif(Source, TargetSource) },
    route_href(RootSource, TargetSource),
    "#",
    seq(Name).

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

html_edges_([edge(publish, Target, Label)|Edges]) -->
    publish_marker(Target, Label),
    html_edges_(Edges).
html_edges_([edge(entry, Target, [])|Edges]) -->
    entry_marker(Target),
    html_edges_(Edges).
html_edges_([edge(link, Target, Label)|Edges]) -->
    link_marker(Target, Label),
    html_edges_(Edges).
html_edges_([edge(label, Name)|Edges]) -->
    label_marker(Name),
    html_edges_(Edges).
html_edges_([edge(label_ref, Name, Label)|Edges]) -->
    label_ref_marker(Name, Label),
    html_edges_(Edges).
html_edges_(Edges) -->
    non_marker_char(_),
    html_edges_(Edges).
html_edges_([]) --> [].

publish_marker(Target, Label) -->
    "<publication-graph-publish data-target=\"",
    attr_value(Target),
    ">",
    publish_body(Label).

entry_marker(Target) -->
    "<publication-graph-entry data-target=\"",
    attr_value(Target),
    "></publication-graph-entry>".

link_marker(Target, Label) -->
    "<publication-graph-link data-target=\"",
    attr_value(Target),
    ">",
    link_body(Label).

label_marker(Name) -->
    "<publication-graph-label data-label=\"",
    attr_value(Name),
    "></publication-graph-label>".

label_ref_marker(Name, Label) -->
    "<publication-graph-ref data-label=\"",
    attr_value(Name),
    ">",
    label_ref_body(Label).

attr_value([]) --> "\"".
attr_value([C|Cs]) --> [C], { dif(C, '"') }, attr_value(Cs).

publish_body([]) --> "</publication-graph-publish>".
publish_body([C|Cs]) --> [C], { dif(C, '<') }, publish_body(Cs).

link_body([]) --> "</publication-graph-link>".
link_body([C|Cs]) --> [C], { dif(C, '<') }, link_body(Cs).

label_ref_body([]) --> "</publication-graph-ref>".
label_ref_body([C|Cs]) --> [C], { dif(C, '<') }, label_ref_body(Cs).

non_marker_char(C) -->
    [C],
    { dif(C, '<') }.
non_marker_char('<') -->
    "<",
    not_marker_prefix.

% Commit point: at "<publication-graph-" the parser MUST match a marker, not
% character-eat the prefix. Without this, html_edges//1 is
% non-deterministic and validation can be bypassed by backtracking into
% an alternative parse where a marker was never extracted.
not_marker_prefix --> not_marker_prefix_after("publication-graph-").

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
