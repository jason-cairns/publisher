#import "/publisher.typ"

== Blog 1

This is my blog. I can have images,
#figure(
  image("../assets/img-1.png", width: 80%),
  caption: [An image of an image.],
)
#link("https://example.com")[links], and all other regular typst things.
I can also add citations @cite, and because this bibliography is scoped to the
current page, it won't include citations from other nodes.

#publisher.bibliography("works.yml", scope: "current-page")
