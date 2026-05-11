#import "_publication.typ": entry, publication-label, publication-link, publication-ref, publish

= cair.nz

#publish("writing.typ")[Writing]

#publish("thesis.typ")[Thesis]

#entry("colophon.typ")

This page is a standalone Typst publication.

#publication-label(<root-note>)

It proves the smallest publishing path: Typst source to intermediate HTML,
transformed site HTML, and a unified PDF artifact.

Read the #publication-link("colophon.typ")[colophon].

The #publication-ref(<root-note>)[root note] is local, and the
#publication-ref(<foo-note>)[foo note] is in another publication.
