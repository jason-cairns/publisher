#import "publisher-nav.typ" as nav

#let marker(kind, fields: (:)) = [
  #metadata(fields + (kind: kind)) <publisher-marker>
]

#let child(path) = marker("child", fields: (path: path))

#let children(pattern) = marker("children", fields: (pattern: pattern))

#let scope(kind: none, name: none) = marker(
  "scope",
  fields: (
    scope_kind: kind,
    name: name,
  ),
)

#let outline(depth: none, title: none, target: none) = marker(
  "outline",
  fields: (
    depth: depth,
    title: title,
    target: target,
  ),
)

#let bibliography(source, scope: none) = marker(
  "bibliography",
  fields: (
    source: source,
    scope: scope,
  ),
)

#let ref(target) = marker("reference", fields: (target: target))
