#let builtin-outline = outline
#import "published.typ"

#let marker(kind, fields: (:)) = [
  #metadata(fields + (kind: kind)) <publisher-marker>
]

#let emit-include(item) = {
  item
}

#let emit-includes(items) = {
  if type(items) == array {
    for item in items {
      emit-include(item)
    }
  } else {
    emit-include(items)
  }
}

#let scope(id, title: none, tags: (), includes: ()) = [
  #marker(
    "scope",
    fields: (
      id: id,
      title: title,
      tags: tags,
    ),
  )
  #emit-includes(includes)
]

#let publish(path) = marker("publish", fields: (path: path))

#let css(path) = marker(
  "payload",
  fields: (
    payload_kind: "css",
    value: path,
  ),
)

#let outline(..args) = {
  let positional = args.pos()
  if positional.len() > 0 {
    let selector = positional.at(0)
    if type(selector) == dictionary and selector.kind == "publisher-selector" {
      return marker(
        "payload",
        fields: (
          payload_kind: "outline",
          value: selector.selector,
          depth: args.named().at("depth", default: none),
        ),
      )
    }
  }
  builtin-outline(..args)
}

#let in-scope(..ids, body) = [
  #metadata((kind: "in-scope", ids: ids.pos())) <publisher-marker>
  #body
]
