@group(0) @binding(0) var<storage, read> firstMatrix : array<u32>;
@group(0) @binding(1) var<storage, read_write> resultMatrix : array<u32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    resultMatrix[global_id.x] = 22u + firstMatrix[global_id.x];
}
