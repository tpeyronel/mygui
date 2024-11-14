struct GlobalUniform {
    viewport_width: f32,
    viewport_height: f32,
}

@group(0) @binding(0) var<uniform> global_uniform: GlobalUniform;

struct VertexIn {
    @location(0) bbox: vec4<f32>,
    @location(1) border_radius: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) pos: vec2<f32>,
}

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) bbox: vec4<f32>,
    @location(3) border_radius: vec4<f32>,
};

const border_width: f32 = 4.0;

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
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let distance = distance(in.pos, vec2<f32>(0.5 * global_uniform.viewport_width, 0.5 * global_uniform.viewport_height));
    let background_color = select(in.color, 1.0 - in.color, 100.0 < distance && distance < 200.0);
    let border_color = vec4(0.0, 0.0, 1.0, 1.0);

    if (in.pos.x < in.bbox.x + in.border_radius.x && in.pos.y < in.bbox.y + in.border_radius.x) {
        let distance_to_corner = distance(in.pos, in.bbox.xy + in.border_radius.x);
        return select(background_color, border_color, in.border_radius.x - border_width < distance_to_corner && distance_to_corner < in.border_radius.x);
    } else if (in.pos.x > in.bbox.z - in.border_radius.y && in.pos.y < in.bbox.y + in.border_radius.y) {
        let distance_to_corner = distance(in.pos, vec2(in.bbox.z - in.border_radius.y, in.bbox.y + in.border_radius.y));
        return select(background_color, border_color, in.border_radius.y - border_width < distance_to_corner && distance_to_corner < in.border_radius.y);
    } else if (in.pos.x > in.bbox.z - in.border_radius.z && in.pos.y > in.bbox.w - in.border_radius.z) {
        let distance_to_corner = distance(in.pos, in.bbox.zw - in.border_radius.z);
        return select(background_color, border_color, in.border_radius.z - border_width < distance_to_corner && distance_to_corner < in.border_radius.z);
    } else if (in.pos.x < in.bbox.x + in.border_radius.w && in.pos.y > in.bbox.w - in.border_radius.w) {
        let distance_to_corner = distance(in.pos, vec2(in.bbox.x + in.border_radius.w, in.bbox.w - in.border_radius.w));
        return select(background_color, border_color, in.border_radius.w - border_width < distance_to_corner && distance_to_corner < in.border_radius.w);
    } else if (in.pos.x > in.bbox.x + border_width && in.pos.x < in.bbox.z - border_width && in.pos.y > in.bbox.y + border_width && in.pos.y < in.bbox.w - border_width) {
        return background_color;
    } else {
        return border_color;
    }


    // if (in.bbox.x + in.border_radius.x < in.pos.x && in.pos.x < in.bbox.z - in.border_radius.x) {
    //     return background_color;
    // }

    // if (in.bbox.y + in.border_radius.y < in.pos.y && in.pos.y < in.bbox.w - in.border_radius.y) {
    //     return background_color;
    // }

    // return border_color;



    // // if (in.pos.x < in.bbox.x + pixel_width || in.pos.y < in.bbox.y + pixel_height) {
    // if (in.pos.x < in.bbox.x + 4.0 * pixel_width
    //     || in.pos.y < in.bbox.y + 4.0 * pixel_height
    //     || in.pos.x > in.bbox.z - 4.0 * pixel_width
    //     || in.pos.y > in.bbox.w - 4.0 * pixel_height) {
    //     return vec4(1.0, 1.0, 1.0, 1.0);
    // }


    // let distance = distance(in.pos, vec2<f32>(0.0, 0.0));
    // return select(in.color, 1.0 - in.color, 0.25 < distance && distance < 0.35  );
    // return vec4(in.pos + 1.0, 0.0, 1.0);
    // return vec4(in.bbox + 1.0, 0.0, 1.0);

}