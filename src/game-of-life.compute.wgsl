@binding(0) @group(0) var<storage, read> current: array<u32>;
@binding(1) @group(0) var<storage, read_write> next: array<u32>;
@binding(2) @group(0) var<uniform> size: vec2<u32>;
@binding(3) @group(0) var<uniform> rule: vec2<i32>;

fn modulo_euclidean(a: i32, b: i32) -> i32 {
    let m = a % b;
    return m + select(0, b, m < 0);
}

fn getIndex(x: i32, y: i32) -> u32 {
    let w = i32(size.x);
    let h = i32(size.y);
    return u32(modulo_euclidean(y, h) * w + modulo_euclidean(x, w));
}

fn getCell(x: i32, y: i32) -> u32 {
    return current[getIndex(x, y)];
}

fn isAlive(x: i32, y: i32) -> u32 {
    return u32(getCell(x, y) > 0u);
}

fn countNeighbors(x: i32, y: i32) -> u32 {
    return isAlive(x - 1, y - 1) + isAlive(x, y - 1) + isAlive(x + 1, y - 1) + isAlive(x - 1, y) + isAlive(x + 1, y) + isAlive(x - 1, y + 1) + isAlive(x, y + 1) + isAlive(x + 1, y + 1);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) grid: vec3<u32>) {
    let x = i32(grid.x);
    let y = i32(grid.y);
    let n = countNeighbors(x, y);
    let current_generation = getCell(x, y);
    let cell_lives = current_generation >= 1u;
    let will_be_born = u32(((1 << n) & rule.x) > 0);
    let will_survive = u32(((1 << n) & rule.y) > 0) * (1u + current_generation);
    next[getIndex(x, y)] = select(will_be_born, will_survive, cell_lives);
} 
