#import publisher

= My Publication

Hello, welcome to my publication!

Here are the contents:

#publisher.outline(depth: 1)

#publisher.child("writing.typ")
#publisher.child("thesis/intro.typ")
#publisher.child("cv.typ")
#publisher.scope(kind: "nav")
#publisher.nav.suppress()
