struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

@group(0) @binding(0) var<uniform> global_uniform: GlobalUniform;
@group(1) @binding(0) var u_texture: texture_2d<f32>;
@group(1) @binding(1) var u_sampler: sampler;
var<push_constant> u_text_color: vec4<f32>;

struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32, in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out._position = vec4<f32>(
        2.0 * in.pos.x / global_uniform.viewport_width - 1.0,
        2.0 * in.pos.y / global_uniform.viewport_height - 1.0,
        0.0,
        1.0,
    );
    out.uv = in.uv;
    return out;
}

struct FragmentOut {
    @location(0) color: vec4<f32>,
    @location(0) @second_blend_source mask: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOut) -> FragmentOut {
    let glyph_alpha = vec4(textureSample(u_texture, u_sampler, in.uv).rgb, 1.0);
    var out: FragmentOut;
    out.color = u_text_color * glyph_alpha;
    out.mask = u_text_color.a * glyph_alpha;
    return out;
}