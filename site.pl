:- use_module(library(dcgs)).
:- use_module(library(lists)).
:- use_module(library(os)).
:- use_module(library(pio)).

:- initialization(main).

main :-
    argv([InputDir, OutputDir]),
    transform(InputDir, OutputDir),
    halt.
main :-
    argv([_, _]),
    write("site transform failed"),
    nl,
    halt(1).
main :-
    write("usage: scryer-prolog site.pl -- INPUT_DIR OUTPUT_DIR"),
    nl,
    halt(1).

transform(InputDir, OutputDir) :-
    path_join(InputDir, "index.html", InputPath),
    path_join(OutputDir, "index.html", OutputPath),
    phrase_from_file(seq(Html), InputPath),
    body_content(Html, Body),
    page(Body, Page, []),
    write_file(OutputPath, Page).

path_join(Dir, File, Path) :-
    append(Dir, "/", Prefix),
    append(Prefix, File, Path).

body_content(Html, Body) :-
    split_after("<body>", Html, AfterBody),
    split_before("</body>", AfterBody, Body0),
    trim_blank_edges(Body0, Body),
    !.
body_content(Html, Html).

split_after(Needle, Haystack, After) :-
    append(_, Rest, Haystack),
    append(Needle, After, Rest).

split_before(Needle, Haystack, Before) :-
    append(Before, Rest, Haystack),
    append(Needle, _, Rest).

trim_blank_edges(In, Out) :-
    trim_left(In, Left),
    reverse(Left, Rev),
    trim_left(Rev, TrimmedRev),
    reverse(TrimmedRev, Out).

trim_left([C|Cs], Out) :-
    blank(C),
    !,
    trim_left(Cs, Out).
trim_left(Cs, Cs).

blank(' ').
blank('\n').
blank('\r').
blank('\t').

page(Body) -->
    "<!doctype html>\n",
    "<html lang=\"en\">\n",
    "  <head>\n",
    "    <meta charset=\"utf-8\">\n",
    "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
    "    <title>cair.nz</title>\n",
    "  </head>\n",
    "  <body>\n",
    "    <header><a href=\"/\">cair.nz</a></header>\n",
    "    <main>\n",
    Body,
    "\n    </main>\n",
    "  </body>\n",
    "</html>\n".

write_file(Path, Chars) :-
    open(Path, write, Stream),
    write_chars(Stream, Chars),
    close(Stream).

write_chars(_, []).
write_chars(Stream, [C|Cs]) :-
    put_char(Stream, C),
    write_chars(Stream, Cs).
