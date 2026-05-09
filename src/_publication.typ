#let nav(dest, body) = context {
  if target() == "html" {
    html.elem("cairnz-nav", attrs: ("data-target": dest), body)
  } else {
    link(dest)[#body]
  }
}

#let publish(dest) = context {
  if target() == "html" {
    html.elem("cairnz-publish", attrs: ("data-target": dest))
  }
}

#let publication-link(dest, body) = context {
  if target() == "html" {
    html.elem("cairnz-link", attrs: ("data-target": dest), body)
  } else {
    link(dest)[#body]
  }
}
