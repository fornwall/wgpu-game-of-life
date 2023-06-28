# wgpu-game-of-life
A demo using the rust [wgpu](https://wgpu.rs/) implementation of the [WebGPU](https://www.w3.org/TR/webgpu/) API to compute and render the [Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) cellular automaton.

<p align="center"><img src="https://wgpu-game-of-life.fornwall.net/static/screenshot.png" width="400"/></p>

See the online version in a WebGPU capable browser at https://wgpu-game-of-life.fornwall.net or run a checkout locally using `cargo run`.

## Keyboard shortcuts
- Change generations per second: `Q` to go slower, `W` to speed up
- Change grid size: `-`/`+`
- Change initial density: `Left`/`Right` arrows
- Change rule: `Up`/`Down` arrows
- Download image: `I` (only works on web)
- Play/Pause: `Space`
- Reset: `R`
- Toggle fullscreen: `F`
