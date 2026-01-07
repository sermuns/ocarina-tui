#import "lib.typ": *

#set page(
  width: 1280pt,
  height: 640pt,
  margin: 100pt,
  fill: background,
)

#set text(
  size: 100pt,
  font: "Charlemagne",
  fill: foreground,
)

#set align(center + horizon)
#set image(height: 1em)
#stack(
  dir: ltr,
  spacing: .5em,
  image("ocarina.svg"),
  [ocarina-tui],
)
