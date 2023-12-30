@binding(0) @group(0) var<storage, read> current: array<u32>;
@binding(1) @group(0) var<storage, read_write> next: array<u32>;
@binding(2) @group(0) var<uniform> size: vec2<u32>;
@binding(3) @group(0) var<uniform> rule: vec2<i32>;

fn modulo_euclidean(a: i32, b: i32) -> i32 {
    let m = a % b;
    return m + select(0, b, m < 0);
}

fn get_index(x: i32, y: i32) -> u32 {
    let w = i32(size.x);
    let h = i32(size.y);
    return u32(modulo_euclidean(y, h) * w + modulo_euclidean(x, w));
}

fn get_cell(x: i32, y: i32) -> u32 {
    return current[get_index(x, y)];
}

fn is_alive(x: i32, y: i32) -> u32 {
    return u32(get_cell(x, y) > 0u);
}

fn count_neighbors(x: i32, y: i32) -> u32 {
    return is_alive(x - 1, y - 1) + is_alive(x, y - 1) + is_alive(x + 1, y - 1) + is_alive(x - 1, y) + is_alive(x + 1, y) + is_alive(x - 1, y + 1) + is_alive(x, y + 1) + is_alive(x + 1, y + 1);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) grid: vec3<u32>) {
    let x = i32(grid.x);
    let y = i32(grid.y);
    let n = count_neighbors(x, y);
    let current_generation = get_cell(x, y);
    let cell_lives = current_generation >= 1u;
    let will_be_born = u32(((1u << n) & u32(rule.x)) > 0u);
    let will_survive = u32(((1u << n) & u32(rule.y)) > 0u) * (1u + current_generation);
    next[get_index(x, y)] = select(will_be_born, will_survive, cell_lives);
} 
