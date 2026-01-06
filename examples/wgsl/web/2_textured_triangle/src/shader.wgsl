struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) texture_coordinates: vec2<f32>
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>
}

@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

@vertex
fn vert_main(vertex: VertexIn) -> VertexOut {
    var result = VertexOut();

    result.position = vec4(vertex.position, 0.0, 1.0);
    result.texture_coordinates = vertex.texture_coordinates;

    return result;
}

@fragment
fn frag_main(@location(0) texture_coordinates: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, texture_coordinates);
}
