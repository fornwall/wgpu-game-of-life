struct Out {
  @builtin(position) pos: vec4<f32>,
  @location(0) cell: f32,
}

@vertex
fn vertex_main(@builtin(instance_index) i: u32, @location(0) cell: u32, @location(1) pos: vec2<u32>) -> Out {
    let w = 256u; // size.x;
    let h = 256u; // size.y;
    let x = (f32(i % w + pos.x) / f32(w) - 0.5) * 2. * f32(w) / f32(max(w, h));
    let y = (f32((i - (i % w)) / w + pos.y) / f32(h) - 0.5) * 2. * f32(h) / f32(max(w, h));

    return Out(vec4<f32>(x, y, 0., 1.), f32(cell));
}

@fragment
fn fragment_main(@location(0) cell: f32) -> @location(0) vec4<f32> {
    return vec4<f32>(cell, cell, cell, 1.);
}
