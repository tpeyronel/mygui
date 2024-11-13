struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOut {
    @builtin(position) _position: vec4<f32>,
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out._position = vec4<f32>(in.pos, 1.0);
    out.pos = in.pos;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let distance = distance(in.pos.xy, vec2<f32>(0.0, 0.0));
    return select(vec4<f32>(in.color, 1.0), vec4(0.0, 1.0, 0.0, 0.0), 0.25 < distance && distance < 0.35  );
}