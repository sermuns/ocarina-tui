#import "lib.typ": *

#set page(
  height: 500pt,
  width: 500pt,
  margin: 50pt,
  fill: none,
  background: box(
    width: 100%,
    height: 100%,
    fill: background,
    radius: 10%,
  ),
)
#set align(center)

#set text(
  size: 200pt,
  font: "Charlemagne",
  fill: foreground,
)

#image("ocarina.svg")
#place(center + horizon, block(
  width: 100%,
  height: 100%,
  fill: background.transparentize(30%),
))
#place(center + horizon)[TUI]
