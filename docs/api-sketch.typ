= Publisher API sketch

This sketch shows the PRD-003 authoring model.
Authors declare publication edges and scopes, while ordinary Typst calls remain ordinary Typst.

= Files

```text
/
  index.typ
  publisher.typ
  writing.typ
  writing/
    blog-1.typ
    blog-2.typ
  thesis/
    intro.typ
    ch-1.typ
  cv.typ
  summary.typ
  works.yml
```

== index.typ

```typ
#import "publisher.typ": scope, publish

= My Publication <home>

Welcome.

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")
#include "summary.typ"

#scope("home", tags: ("nav",))
```

== writing.typ

```typ
#import "publisher.typ": scope, publish, in-scope

= Writing <writing>

#scope("writing", title: [Writing], tags: ("nav",))

#publish("writing/blog-1.typ")
#publish("writing/blog-2.typ")

#outline(target: heading.where(level: 1))

#in-scope("writing", "thesis")[
  #outline(target: heading.where(level: 1))
]
```

== writing/blog-1.typ

```typ
#import "../publisher.typ": scope

= Blog One <blog-one>

This article cites @web.

#bibliography("../works.yml")
```

== writing/blog-2.typ

```typ
#import "../publisher.typ": scope

= Blog Two <blog-two>

This post refers to thesis material at @thesis-main.
```

== thesis/intro.typ

```typ
#import "../publisher.typ": scope, publish

= Thesis Introduction <thesis-intro>

#scope("thesis", title: [Thesis], tags: ("nav", "bibliography"))

#publish("thesis/ch-1.typ")

This source cites @book.

#outline(target: heading.where(level: 1))
#bibliography("../works.yml")
```

== thesis/ch-1.typ

```typ
#import "../publisher.typ": scope

= Chapter One <thesis-main>

The first chapter cites @article.

#bibliography("../works.yml")
```

== cv.typ

```typ
#import "publisher.typ": scope

= CV <cv>

#scope("cv", title: [CV], tags: ("nav",))

about me
```

= Publisher behavior

- `#publish(path)` creates a routed publication edge.
- `#include(path)` remains normal Typst inclusion and does not create a routed publication edge.
- `#scope(id, title: none, tags: ())` declares a publisher scope over the source document and its published descendants.
- Ordinary `#outline(...)`, `#bibliography(...)`, `@label` references, counters, and `query(...)` stay authored as ordinary Typst.
- The publisher may generate scoped Typst entrypoints or per-source HTML adapters before compilation, but those adapters are not author-facing APIs.

= CLI sketch

```sh
publisher build index.typ --html --pdf
publisher inspect index.typ
publisher render-scope index.typ thesis --pdf
publisher render-union index.typ writing thesis --pdf
```
