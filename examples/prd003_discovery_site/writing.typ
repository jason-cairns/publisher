#import "publisher.typ": scope, publish, in-scope

= Writing Landing <writing-landing>

#scope("writing", title: [Writing], tags: ("nav",))

#publish("writing/blog-1.typ")
#publish("writing/blog-2.typ")

== Local Writing Notes <writing-notes>

#outline(target: heading.where(level: 1))

#in-scope("writing", "thesis")[
  = Cross Scope Outline Probe
  #outline(target: heading.where(level: 1))
]
