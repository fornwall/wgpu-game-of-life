fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pollster::block_on(wgpu_game_of_life::run());
}
