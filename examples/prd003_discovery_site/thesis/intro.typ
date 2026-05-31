#import "../publisher.typ": scope, publish

= Thesis Introduction <thesis-intro>

#scope("thesis", title: [Thesis], tags: ("nav", "bibliography"))

#publish("thesis/ch-1.typ")

This source cites @book.

#outline(target: heading.where(level: 1))
#bibliography("../works.yml")
