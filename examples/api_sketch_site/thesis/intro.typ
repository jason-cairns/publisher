#import "/publisher.typ"

= Thesis

#publisher.children("thesis/*.typ")
#publisher.scope(kind: "outline")
#publisher.scope(kind: "reference")
#publisher.scope(kind: "bibliography")

#publisher.outline(depth: 1)
#publisher.outline(
  title: [List of Figures],
  target: figure.where(kind: image),
)
#publisher.scope(kind: "nav")
