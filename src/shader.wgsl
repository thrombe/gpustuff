

// Vertex shader

// struct VertexOutput {
//     [[builtin(position)]] clip_position: vec4<f32>;
// };

// [[stage(vertex)]]
// fn main( [[builtin(vertex_index)]] in_vertex_index: u32 ) -> VertexOutput {
//     var out: VertexOutput;
//     let x = f32(1 - i32(in_vertex_index)) * 0.5;
//     let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
//     out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
//     return out;
// }

// Fragment shader

[[block]]
struct Stuff {
    time: f32;
    vare: f32;
};
[[group(0), binding(0)]]
var<uniform> stuff: Stuff;

// [[stage(fragment)]]
// fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
//     return vec4<f32>(stuff.time, in.clip_position[0], in.clip_position[1], 1.0);
// }




// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.clip_position);
}