# wgpu-game-of-life

A demo using the rust [wgpu](https://wgpu.rs/) implementation of the [WebGPU](https://www.w3.org/TR/webgpu/) API to compute and render the [Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) and [Life-like](https://conwaylife.com/wiki/Life-like_cellular_automaton) simulations.

See the online version in a WebGPU capable browser at https://wgpu-game-of-life.fornwall.net or run a checkout locally using `cargo run`.

<p align="center"><picture><img src="site/static/screenshot.png" width="300"/></picture></p>

It's based on [WebGPU Samples](https://webgpu.github.io/webgpu-samples/samples/gameOfLife), ported to rust and `wgpu` (with the help of [Learn Wgpu](https://sotrh.github.io/learn-wgpu/)) and having some additional features for demonstration purposes. It uses [winit](https://github.com/rust-windowing/winit) for cross-platform windowing functionality.

## Keyboard shortcuts

- Change generations per second: `Q` to go slower, `W` to speed up
- Change grid size: `-`/`+`
- Change initial density: `Left`/`Right` arrows
- Change rule: `Up`/`Down` arrows
- Download image: `I` (web version only)
- Play/Pause: `Space`
- Reset: `R`
- Toggle controls: `Tab`/`C` (web version only)
- Toggle fullscreen: `F`

## Platform: Android

Use [cargo-ndk](https://crates.io/crates/cargo-ndk) to build and run (release builds requires the `CARGO_APK_RELEASE_KEYSTORE` and `CARGO_APK_RELEASE_KEYSTORE_PASSWORD` environment variables to be set).

- `make [RELEASE=1] build-android`: Build apk at `target/[debug/release]/apk/wgpu-game-of-life.apk`
- `make [RELEASE=1] run-android`: Run app on connected device or emulator.

## Platform: iOS

Run `make run-ios-simulator` to run inside the iOS simulator.

## Platform: Web

Run `make run-web` to build, serve and open the web version in a browser.

## Resources

- [Game of Life: How a nerdsnipe led to a fast implementation of game of life](https://binary-banter.github.io/game-of-life/)
