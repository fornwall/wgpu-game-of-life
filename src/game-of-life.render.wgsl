struct Out {
  @builtin(position) pos: vec4<f32>,
  @location(0) cell: vec3<f32>,
}

@binding(0) @group(0) var<uniform> size: vec2<u32>;

@vertex
fn vertex_main(@builtin(instance_index) i: u32, @location(0) cell: u32, @location(1) pos: vec2<u32>) -> Out {
    let w = size.x;
    let h = size.y;
    let x = (f32(i % w + pos.x) / f32(w) - 0.5) * 2. * f32(w) / f32(max(w, h));
    let y = (f32((i - (i % w)) / w + pos.y) / f32(h) - 0.5) * 2. * f32(h) / f32(max(w, h));

    let c = f32(i) / f32(size.x * size.y);
    let r = f32(cell) * f32(i % w) / f32(w);
    let g = f32(cell) * f32(i / w) / f32(h);
    let b = f32(cell) * (1. - max(g, r));
    return Out(vec4<f32>(x, y, 0., 1.), vec3(r, g, b));
}

@fragment
fn fragment_main(@location(0) cell: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(cell, 1.0);
}
