#let publication_target(dest) = "publication:" + dest
#let html_export = sys.inputs.at("render-target", default: "paged") == "html"

#let publication_target_link(dest, body) = {
  let target = label(publication_target(dest))
  if query(target).len() > 0 {
    link(target)[#body]
  } else {
    link(dest)[#body]
  }
}

#let publish(dest, body) = context {
  if html_export {
    html.elem("publication-publish", attrs: ("data-target": dest), body)
  } else {
    publication_target_link(dest, body)
  }
}

#let entry(dest) = context {
  if html_export {
    html.elem("publication-entry", attrs: ("data-target": dest))
  }
}

#let publication-link(dest, body) = context {
  if html_export {
    html.elem("publication-link", attrs: ("data-target": dest), body)
  } else {
    publication_target_link(dest, body)
  }
}

#let publication(dest, body) = context [
  #metadata(none) #label(publication_target(dest))
  #body
]
