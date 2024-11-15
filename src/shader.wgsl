struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

@group(0) @binding(0) var<uniform> global_uniform: GlobalUniform;

struct VertexIn {
    @location(0) bbox: vec4<f32>,
    @location(1) border_radius: vec4<f32>,
    @location(2) border_width: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) pos: vec2<f32>,
}

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) bbox: vec4<f32>,
    @location(3) border_radius: vec4<f32>,
    @location(4) border_width: vec4<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out._position = vec4<f32>(
        2.0 * in.pos.x / global_uniform.viewport_width - 1.0,
        2.0 * in.pos.y / global_uniform.viewport_height - 1.0,
        0.0,
        1.0,
    );
    out.pos = in.pos;
    out.color = in.color;
    out.bbox = in.bbox;
    out.border_radius = in.border_radius;
    out.border_width = in.border_width;
    return out;
}

fn square(v: f32) -> f32 { return v * v; }

fn double_smoothstep(edge0: f32, edge1: f32, edge2: f32, edge3: f32, value: f32) -> f32 {
    if (value <= edge1) {
        return smoothstep(edge0, edge1, value);
    } else {
        return smoothstep(edge3, edge2, value);
    }
}

fn smooth_corner(
    frag_pos: vec2<f32>,
    corner_pos: vec2<f32>,
    radius: f32,
    thickness: vec2<f32>,
    background_color: vec4<f32>,
    border_color: vec4<f32>,
) -> vec4<f32> {
    let corner_to_frag = frag_pos - corner_pos;
    let distance_to_corner = length(corner_to_frag);
    let mixed_thickness = mix(thickness.y, thickness.x, square(abs(corner_to_frag.x) / distance_to_corner));

    let delta = fwidth(distance_to_corner) * 0.3;

    let outer_radius = radius;
    let inner_radius = outer_radius - mixed_thickness;

    if (distance_to_corner >= outer_radius - delta) {
        let alpha = smoothstep(outer_radius - delta, outer_radius + delta, distance_to_corner);
        return mix(border_color, vec4(0.0), alpha);
    } else {
        let alpha = smoothstep(inner_radius - delta, inner_radius + delta, distance_to_corner);
        return mix(background_color, border_color, alpha);
    }
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let center = vec2(0.5 * global_uniform.viewport_width, 0.5 * global_uniform.viewport_height);
    let distance = distance(in.pos, center);
    let delta = fwidth(distance);
    let alpha = double_smoothstep(80.0 - delta, 80.0, 100.0 - delta, 100.0, distance);
    // let background_color = mix(in.color, 1.0 - in.color, alpha);
    let background_color = vec4(0.3);
    let border_color = vec4(0.0, 1.0, 0.0, 0.0);

    if (in.pos.x < in.bbox.x + in.border_radius.x && in.pos.y < in.bbox.y + in.border_radius.x) {
        return smooth_corner(in.pos, in.bbox.xy + in.border_radius.x, in.border_radius.x, in.border_width.wx, background_color, border_color);
    } else if (in.pos.x > in.bbox.z - in.border_radius.y && in.pos.y < in.bbox.y + in.border_radius.y) {
        return smooth_corner(in.pos, vec2(in.bbox.z - in.border_radius.y, in.bbox.y + in.border_radius.y), in.border_radius.y, in.border_width.yx, background_color, border_color);
    } else if (in.pos.x > in.bbox.z - in.border_radius.z && in.pos.y > in.bbox.w - in.border_radius.z) {
        return smooth_corner(in.pos, in.bbox.zw - in.border_radius.z, in.border_radius.z, in.border_width.yz, background_color, border_color);
    } else if (in.pos.x < in.bbox.x + in.border_radius.w && in.pos.y > in.bbox.w - in.border_radius.w) {
        return smooth_corner(in.pos, vec2(in.bbox.x + in.border_radius.w, in.bbox.w - in.border_radius.w), in.border_radius.w, in.border_width.wz, background_color, border_color);
    } else if (in.pos.y < in.bbox.y + in.border_width.x || in.pos.x > in.bbox.z - in.border_width.y || in.pos.y > in.bbox.w - in.border_width.z || in.pos.x < in.bbox.x + in.border_width.w) {
        return border_color;
    } else {
        return background_color;
    }
}