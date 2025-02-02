@group(0) @binding(0) var framebuffer_image: texture_2d<f32>;
@group(0) @binding(1) var framebuffer_sampler: sampler;

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOut {
    let uv = vec2<f32>(f32((vertex_idx << 1) & 2), f32(vertex_idx & 2));

    var out: VertexOut;
    out._position = vec4<f32>(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2(uv.x, 1.0 - uv.y);
    return out;
}

fn linear_to_srgb(c: f32) -> f32 {
    if c < 0.0031308 {
        return c * 12.92;
    } else {
        return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
    }
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let pixel = textureSample(framebuffer_image, framebuffer_sampler, in.uv);

    // let corrected_rgb = 1.055 * pow(pixel.rgb, vec3(1.0 / 2.4)) - 0.055;

    let corrected_rgb = vec3(
        linear_to_srgb(pixel.r),
        linear_to_srgb(pixel.g),
        linear_to_srgb(pixel.b),
    );

    return vec4(corrected_rgb, pixel.a);
}
