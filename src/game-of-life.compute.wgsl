@binding(0) @group(0) var<storage, read> current: array<u32>;
@binding(1) @group(0) var<storage, read_write> next: array<u32>;

fn getIndex(x: i32, y: i32) -> u32 {
    let h = 256; // TODO: Should come from override, must mach lib.rs
    let w = 256; // TODO: Should come from override, must match lib.rs

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
    next[getIndex(x, y)] = select(u32(n == 3u), u32(n == 2u || n == 3u), getCell(x, y) == 1u);
} 
