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

fn square(v: f32) -> f32 { return v * v; }

fn smooth_corner(
    frag_pos: vec2<f32>,
    corner_pos: vec2<f32>,
    radius: f32,
    thickness: vec2<f32>,
    fill_color: vec4<f32>,
    border_color: vec4<f32>,
) -> vec4<f32> {
    let corner_to_frag = frag_pos - corner_pos;
    let distance_to_corner = length(corner_to_frag);
    let mixed_thickness = mix(thickness.y, thickness.x, square(abs(corner_to_frag.x) / distance_to_corner));

    // let delta = fwidth(distance_to_corner) * 0.3; // <- this doesn't work correctly for fragments that are at the edge of the corner
    let delta = 0.5;

    let outer_radius = radius;
    let inner_radius = outer_radius - mixed_thickness;

    if (outer_radius <= inner_radius) {
        let alpha = smoothstep(outer_radius - delta, outer_radius + delta, distance_to_corner);
        return mix(fill_color, vec4(0.0), alpha);
    } if (distance_to_corner >= outer_radius - delta) {
        let alpha = smoothstep(outer_radius - delta, outer_radius + delta, distance_to_corner);
        return mix(border_color, vec4(0.0), alpha);
    } else {
        let alpha = smoothstep(inner_radius - delta, inner_radius + delta, distance_to_corner);
        return mix(fill_color, border_color, alpha);
    }
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let data = rectangle_data[rectangle_idx];

    let fill_color = data.fill_color;
    let border_color = data.border_color;
    let bbox = vec4(data.left, data.bottom, data.right, data.top);

    var color: vec4<f32> = vec4(0.0);

    if (in.pos.x < bbox.x + data.border_radius.x && in.pos.y < bbox.y + data.border_radius.x) { // bottom-left corner
        color += smooth_corner(in.pos, bbox.xy + data.border_radius.x, data.border_radius.x, data.border_width.xy, fill_color, border_color);
    } else if (in.pos.x > bbox.z - data.border_radius.y && in.pos.y < bbox.y + data.border_radius.y) { // bottom-right corner
        color += smooth_corner(in.pos, vec2(bbox.z - data.border_radius.y, bbox.y + data.border_radius.y), data.border_radius.y, data.border_width.zy, fill_color, border_color);
    } else if (in.pos.x > bbox.z - data.border_radius.z && in.pos.y > bbox.w - data.border_radius.z) { // top-right corner
        color += smooth_corner(in.pos, bbox.zw - data.border_radius.z, data.border_radius.z, data.border_width.zw, fill_color, border_color);
    } else if (in.pos.x < bbox.x + data.border_radius.w && in.pos.y > bbox.w - data.border_radius.w) { // top-left corner
        color += smooth_corner(in.pos, vec2(bbox.x + data.border_radius.w, bbox.w - data.border_radius.w), data.border_radius.w, data.border_width.xw, fill_color, border_color);
    } else if (in.pos.y < bbox.y + data.border_width.y || in.pos.x > bbox.z - data.border_width.z || in.pos.y > bbox.w - data.border_width.w || in.pos.x < bbox.x + data.border_width.x) {
        color += border_color;
    } else {
        color += fill_color;
    }

    return color;
}