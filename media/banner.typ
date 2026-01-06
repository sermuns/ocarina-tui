#import "lib.typ": *

#set page(
  width: 1073pt,
  height: 151pt,
  margin: (x: 4%),
  fill: none,
  background: box(
    width: 100%,
    height: 100%,
    fill: background,
    radius: 10%,
  ),
)

#set text(
  size: 90pt,
  font: "Charlemagne",
  fill: foreground,
)

#set align(center + horizon)
#set image(height: 90pt)
#stack(
  dir: ltr,
  spacing: 1fr,
  image("ocarina.svg"),
  [ocarina-tui],
  image("ocarina.svg"),
)
