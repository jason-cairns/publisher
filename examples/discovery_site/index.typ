#import "publisher.typ": scope, publish

= Home <home-heading>

Welcome to the discovery publication.

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")
#include "summary.typ"

#scope("home", tags: ("nav",))
