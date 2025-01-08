struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

struct RectangleData {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
    fill_color: vec4<f32>,
    border_color: vec4<f32>,
    border_radius: vec4<f32>,
    border_width: vec4<f32>, // left, bottom, right, top
}

@group(0) @binding(0) var<uniform> global_uniform: GlobalUniform;
@group(1) @binding(0) var<storage, read> rectangle_data: array<RectangleData>;
var<push_constant> rectangle_idx: u32;

struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) pos: vec2<f32>,
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
    out.pos = in.pos;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let data = rectangle_data[rectangle_idx];

    let fill_color = data.fill_color;
    let border_color = data.border_color;

    return fill_color;
}