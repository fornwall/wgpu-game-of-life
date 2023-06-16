@binding(0) @group(0) var<storage, read> current: array<u32>;
@binding(1) @group(0) var<storage, read_write> next: array<u32>;
@binding(2) @group(0) var<storage, read> size: vec2<u32>;
@binding(3) @group(0) var<storage, read> rule: vec2<i32>;

fn getIndex(x: i32, y: i32) -> u32 {
    let w = i32(size.x);
    let h = i32(size.y);
    return u32((y % h) * w + (x % w));
}

fn getCell(x: i32, y: i32) -> u32 {
    return current[getIndex(x, y)];
}

fn countNeighbors(x: i32, y: i32) -> u32 {
    return getCell(x - 1, y - 1) + getCell(x, y - 1) + getCell(x + 1, y - 1) + getCell(x - 1, y) + getCell(x + 1, y) + getCell(x - 1, y + 1) + getCell(x, y + 1) + getCell(x + 1, y + 1);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) grid: vec3<u32>) {
    let x = i32(grid.x);
    let y = i32(grid.y);
    let n = countNeighbors(x, y);
    let will_be_born = u32(((1 << n) & rule.x) > 0);
    let will_survive = u32(((1 << n) & rule.y) > 0);
    let cell_lives = getCell(x, y) == 1u;
    next[getIndex(x, y)] = select(will_be_born, will_survive, cell_lives);
} 
