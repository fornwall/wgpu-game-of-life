struct Out {
  @builtin(position) pos: vec4<f32>,
  @location(0) cell: vec3<f32>,
}

@binding(0) @group(0) var<uniform> size: vec2<u32>;

@vertex
fn vertex_main(@builtin(instance_index) i: u32, @location(0) cell: u32, @location(1) pos: vec2<u32>) -> Out {
    let w = size.x;
    let h = size.y;
    let x = (f32(i % w + pos.x) / f32(w) - 0.5) * 2. * f32(w) / f32(w);
    let y = (f32((i - (i % w)) / w + pos.y) / f32(h) - 0.5) * 2. * f32(h) / f32(h);

    let max_age_for_color: u32 = 40u;
    let intensity = 1.0 - f32(min(cell, max_age_for_color)) / f32(max_age_for_color);

    let rgb = select(vec3(0., 0., 0.), spectral_bruton(intensity), cell > 0u);
    return Out(vec4<f32>(x, y, 0., 1.), rgb);
}

@fragment
fn fragment_main(@location(0) cell: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(cell, 1.0);
}

fn spectral_bruton(w: f32) -> vec3<f32> {
    if w < 0.15 {
        return vec3<f32>(0.0, 1.0, -(w - 0.325) / (0.325 - 0.275));
    } else if w >= 0.15 && w < 0.275 {
        return vec3<f32>(0.0, 1.0, -(w - 0.325) / (0.325 - 0.275));
    } else if w >= 0.275 && w < 0.325 {
        return vec3<f32>(0.0, 1.0, -(w - 0.325) / (0.325 - 0.275));
    } else if w >= 0.325 && w < 0.5 {
        return vec3<f32>((w - 0.325) / (0.5 - 0.325), 1.0, 0.0);
    } else if w >= 0.5 {
        return vec3<f32>(1.0, -(w - 0.6625) / (0.6625 - 0.5), 0.0);
    } else {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
}
