# wgpu-game-of-life
A demo using the [wgpu](https://wgpu.rs/) implementation of the [WebGPU](https://www.w3.org/TR/webgpu/) API to compute and render the [Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) cellular automaton.

See the online version in a WebGPU capable browser at https://wgpu-game-of-life.fornwall.net/, or run it locally using `cargo run`.

## Creating a macOS app bundle
Use [cargo-bundle](https://github.com/burtonageo/cargo-bundle):

```sh
cargo install cargo-bundle
cargo bundle
```

This looks at the `package.metadata.bundle` section of Cargo.toml.
