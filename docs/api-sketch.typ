#import "@preview/dtree:0.1.1": dtree

This gives my notion of how the API should look, with an example site given.

The example file structure is as follows.
Note how the directory structure is not relevant, but each source file represents a node.

= Files

#dtree(```
/
 index.typ
 writing.typ
 writing
  blog-1.typ
  blog-2.typ
 thesis
  intro.typ
  ch-1.typ
  bibliography.bib
 cv.typ
 assets
  img-1.jpg
  img-1.png
  img-2.svg
 works.yml
 README.md
```)

== index.typ

```typ
#import publisher

= My Publication

Hello, welcome to my publication!

Here are the contents:

#publisher.outline(depth: 1) // produces an outline of children. Titles taken from their properties.

#publisher.child("writing.typ")
#publisher.child("thesis/intro.typ")
#publisher.child("cv.typ")
#let nav = publisher.scope(kind: "nav") // creates a new "nav" scope. This page and all children will have a nav bar.
#nav.suppress() // suppress the nav for this page. Child pages will still have one.
```

== writing.typ

```typ
#import publisher

= Writing // implicitly creates a title property with value "Writing"

#publisher.children("writing/*.typ") // declares children by file glob
#publisher.scope(kind: "outline") // Creates a new scope for all contents, shadowing parent scope, so only writing contents will be shown.
#publisher.scope(kind: "reference") // "name" field defaults to page title

// multiple scopes, could be #publisher.scope(kind: ("outline", "reference"))
// Or even some wrapper for common scope pairings.

#publisher.outline(depth: 1) // not shown, but default scope is just "nearest". Can be global too.
```

== writing/blog-1.typ

```typ
#import publisher

== Blog 1

This is my blog. I can have images,
#figure(
  image("img-1.jpg", width: 80%),
  caption: [An image of an image.],
)
#link("https://example.com")[links], and all other regular typst things.
I can also add citations @cite, and because this bibliography is scoped to the
current page, it won't include citations from other nodes.

#publisher.bibliography("works.yml", scope: "current-page")
```

== writing/blog-2.typ

```typ
#import publisher

== Blog 2

This blog shows how I can reference labels in other locations.
I can reference #publisher.ref(<figure-1>) from the thesis, and it will show up here
as, "Figure 1, Thesis", where the "Figure 1" text is how it would
typically render, and ", Thesis" is added from the reference scope
it is found under.
```

== thesis/intro.typ

```typ
#import publisher

= Thesis

#publisher.children("thesis/*.typ") // declares children by file glob (excluding itself)
#publisher.scope(kind: "outline") // Creates a new scope for all outlines, shadowing parent scope, so only thesis contents will be shown.
#publisher.scope(kind: "reference") // "name" field defaults to page title
#publisher.scope(kind: "bibliography")

#publisher.outline(depth: 1) // not shown, but default scope is just "nearest". Can be global too.
#publisher.outline( // outline follows the standard typst api. (all publisher apis do.)
  title: [List of Figures],
  target: figure.where(kind: image),
)
#publisher.scope(kind: "nav") // creates a new "nav" scope. This page and all children will have a nav bar. This is a second nav bar, below the inherited nav bar.
```

== thesis/ch-1.typ

```typ
#import publisher

@bib-ref
```

== thesis/bib

```typ
#publisher.bibliography("bibliography.bib") // defaults to nearest scope (thesis)
```

== cv.typ

```typ
#nav.suppress()

about me

etc.
```

= CLI

```sh
publisher --help
  usage:
    publisher [root node] -T[html|pdf]
    publisher dot [tree|graph] --incl-external
    publisher inspect [nodes|scopes|...]
    publisher install-completions [fish|sh|zsh]
```
