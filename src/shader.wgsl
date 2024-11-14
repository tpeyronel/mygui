struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) bbox_bottom_left: vec2<f32>,
    @location(3) bbox_top_right: vec2<f32>,
}

const pixel_width: f32 = 2.0 / 800.0;
const pixel_height: f32 = 2.0 / 600.0;

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) bbox: vec4<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out._position = vec4<f32>(in.pos, 0.0, 1.0);
    out.pos = in.pos;
    out.color = in.color;
    out.bbox = vec4(in.bbox_bottom_left, in.bbox_top_right);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // // if (in.pos.x < in.bbox.x + pixel_width || in.pos.y < in.bbox.y + pixel_height) {
    if (in.pos.x < in.bbox.x + 4.0 * pixel_width
        || in.pos.y < in.bbox.y + 4.0 * pixel_height
        || in.pos.x > in.bbox.z - 4.0 * pixel_width
        || in.pos.y > in.bbox.w - 4.0 * pixel_height) {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }


    let distance = distance(in.pos, vec2<f32>(0.0, 0.0));
    return select(in.color, 1.0 - in.color, 0.25 < distance && distance < 0.35  );
    // return vec4(in.pos + 1.0, 0.0, 1.0);
    // return vec4(in.bbox + 1.0, 0.0, 1.0);

}