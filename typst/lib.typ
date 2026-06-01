#let marker(kind, fields: (:)) = [
  #metadata(fields + (kind: kind)) <publisher-marker>
]

#let scope(id, title: none, tags: ()) = marker(
  "scope",
  fields: (
    id: id,
    title: title,
    tags: tags,
  ),
)

#let publish(path) = marker("publish", fields: (path: path))

#let css(path) = marker(
  "payload",
  fields: (
    payload_kind: "css",
    value: path,
  ),
)

#let in-scope(..ids, body) = [
  #metadata((kind: "in-scope", ids: ids.pos())) <publisher-marker>
  #body
]
