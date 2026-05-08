:- use_module(library(dcgs)).
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
    directory_file_path(InputDir, "index.html", InputPath),
    directory_file_path(OutputDir, "index.html", OutputPath),
    phrase_from_file(seq(Html), InputPath),
    html_site_page(Html, Page),
    write_file(OutputPath, Page).

directory_file_path(Directory, File, Path) :-
    phrase(directory_file_path_(Directory, File), Path).

directory_file_path_(Directory, File) -->
    seq(Directory),
    "/",
    seq(File).

html_site_page(Html, Page) :-
    phrase(html_body(Body), Html),
    phrase(site_page(Body), Page).

html_body(Body) -->
    any_chars,
    "<body>",
    body_chars(Body),
    "</body>",
    any_chars.

any_chars --> [].
any_chars --> [_], any_chars.

body_chars([]) --> [].
body_chars([C|Cs]) --> [C], body_chars(Cs).

site_page(Body) -->
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
