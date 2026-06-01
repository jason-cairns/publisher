= Publisher API sketch

This sketch shows the current authoring model.
Authors declare publication edges and scopes, while ordinary Typst calls remain ordinary Typst.

= Files

```text
/
  index.typ
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
#import "@local/publisher:0.1.0": scope, publish, css

= My Publication <home>

Welcome.

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")
#include "summary.typ"

#scope("home", tags: ("nav",))
#css("styles/site.css")
```

== writing.typ

```typ
#import "@local/publisher:0.1.0": scope, publish, in-scope

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
#import "@local/publisher:0.1.0": scope

= Blog One <blog-one>

This article cites @web.

#bibliography("../works.yml")
```

== writing/blog-2.typ

```typ
#import "@local/publisher:0.1.0": scope

= Blog Two <blog-two>

This post refers to thesis material at @thesis-main.
```

== thesis/intro.typ

```typ
#import "@local/publisher:0.1.0": scope, publish

= Thesis Introduction <thesis-intro>

#scope("thesis", title: [Thesis], tags: ("nav", "bibliography"))

#publish("thesis/ch-1.typ")

This source cites @book.

#outline(target: heading.where(level: 1))
#bibliography("../works.yml")
```

== thesis/ch-1.typ

```typ
#import "@local/publisher:0.1.0": scope

= Chapter One <thesis-main>

The first chapter cites @article.

#bibliography("../works.yml")
```

== cv.typ

```typ
#import "@local/publisher:0.1.0": scope

= CV <cv>

#scope("cv", title: [CV], tags: ("nav",))

about me
```

= Publisher behavior

- `#publish(path)` creates a routed publication edge.
- `#include(path)` remains normal Typst inclusion and does not create a routed publication edge.
- `#scope(id, title: none, tags: ())` declares a publisher scope over the source document and its published descendants.
- `#css("path.css")` attaches a CSS file path to the nearest preceding scope in that source; HTML pages covered by the scope include the CSS in scope-spine order.
- Ordinary `#outline(...)`, `#bibliography(...)`, `@label` references, counters, and `query(...)` stay authored as ordinary Typst.
- The publisher may generate scoped Typst entrypoints or per-source HTML adapters before compilation, but those adapters are not author-facing APIs.

= CLI sketch

```sh
publisher build index.typ --html --pdf
publisher inspect index.typ
publisher render-scope index.typ thesis --pdf
publisher render-union index.typ writing thesis --pdf
```
