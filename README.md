<a href="https://github.com/sermuns/ocarina-tui">
  <img src="media/banner.png">
</a>

<div align="center">
  <p>
  <em>
You found the Ocarina of Time! This is the Royal Family's hidden treasure which Zelda left behind.
  </em>
    </p>
  <a href="https://github.com/sermuns/ocarina-tui/releases/latest">
    <img alt="release-badge" src="https://img.shields.io/github/v/release/sermuns/ocarina-tui.svg"></a>
  <a href="https://github.com/sermuns/ocarina-tui/blob/main/LICENSE">
    <img alt="WTFPL" src="https://img.shields.io/badge/License-WTFPL-brightgreen.svg"></a>
  <a href="https://crates.io/crates/ocarina-tui"><img src="https://img.shields.io/crates/v/ocarina-tui.svg" alt="Version info"></a>
</div>
<br>

A TUI application cooked up with the crates Ratatui[^2] and RustySynth[^3].

With this TUI, you can play the Ocarina, just like in Ocarina of Time and Majora's Mask.

![screenshot](media/screenshot.jpg)

## Features

- Play the five notes in the range D4-D5, just as in-game (no [pitch-bending](<https://zeldawiki.wiki/wiki/Ocarina_of_Time_(Item)#Changing_the_Pitches>) implemented _yet_).
- Get visual and auditory confirmation when you play a song from Ocarina of Time or Majora's Mask. [^4]

## Installation

- from source:
  ```sh
  cargo install ocarina-tui
  ```

- from binaries, using [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):
  ```sh
  cargo binstall ocarina-tui
  ```

- or download the [latest release](https://github.com/sermuns/ocarina-tui/releases/latest)

## Acknowledgments

This project would not have been possible without

- Mathew Valente's recreation of the OoT soundfont [^1]
- The excellent crates Ratatui [^2] and RustySynth [^3]
- Koji Kondo and Nintendo for making such banger soundtrack and game

[^1]: http://tssf.gamemusic.ca/Remakes/index.php?folder=WmVsZGE2NFN0dWZm

[^2]: https://github.com/ratatui/ratatui

[^3]: https://github.com/sinshu/rustysynth

[^4]: though many songs are yet to be implemented...
