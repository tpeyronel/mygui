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

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let distance = distance(in.pos, vec2<f32>(0.5 * global_uniform.viewport_width, 0.5 * global_uniform.viewport_height));
    let background_color = select(in.color, 1.0 - in.color, 100.0 < distance && distance < 200.0);
    let border_color = vec4(0.0, 0.0, 1.0, 1.0);

    if (in.pos.x < in.bbox.x + in.border_radius.x && in.pos.y < in.bbox.y + in.border_radius.x) {
        let corner_to_frag = in.pos - (in.bbox.xy + in.border_radius.x);
        let distance_to_corner = length(corner_to_frag);
        let border_width = mix(in.border_width.w, in.border_width.x, square(-corner_to_frag.y / distance_to_corner));
        return select(background_color, border_color, in.border_radius.x - border_width < distance_to_corner && distance_to_corner < in.border_radius.x);
    } else if (in.pos.x > in.bbox.z - in.border_radius.y && in.pos.y < in.bbox.y + in.border_radius.y) {
        let corner_to_frag = in.pos - vec2(in.bbox.z - in.border_radius.y, in.bbox.y + in.border_radius.y);
        let distance_to_corner = length(corner_to_frag);
        let border_width = mix(in.border_width.x, in.border_width.y, square(corner_to_frag.x / distance_to_corner));
        return select(background_color, border_color, in.border_radius.y - border_width < distance_to_corner && distance_to_corner < in.border_radius.y);
    } else if (in.pos.x > in.bbox.z - in.border_radius.z && in.pos.y > in.bbox.w - in.border_radius.z) {
        let corner_to_frag = in.pos - (in.bbox.zw - in.border_radius.z);
        let distance_to_corner = length(corner_to_frag);
        let border_width = mix(in.border_width.y, in.border_width.z, square(corner_to_frag.y / distance_to_corner));
        return select(background_color, border_color, in.border_radius.z - border_width < distance_to_corner && distance_to_corner < in.border_radius.z);
    } else if (in.pos.x < in.bbox.x + in.border_radius.w && in.pos.y > in.bbox.w - in.border_radius.w) {
        let corner_to_frag = in.pos - vec2(in.bbox.x + in.border_radius.w, in.bbox.w - in.border_radius.w);
        let distance_to_corner = length(corner_to_frag);
        let border_width = mix(in.border_width.z, in.border_width.w, square(-corner_to_frag.x / distance_to_corner));
        return select(background_color, border_color, in.border_radius.w - border_width < distance_to_corner && distance_to_corner < in.border_radius.w);
    } else if (in.pos.y < in.bbox.y + in.border_width.x || in.pos.x > in.bbox.z - in.border_width.y || in.pos.y > in.bbox.w - in.border_width.z || in.pos.x < in.bbox.x + in.border_width.w) {
        return border_color;
    } else {
        return background_color;
    }
}