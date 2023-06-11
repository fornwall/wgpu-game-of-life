fn main() {
    pollster::block_on(wgpu_game_of_life::run());
}
