
// / import ./src/vertex.wgsl

[[stage(vertex)]]
fn main_vertex() -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(1.0);
}


[[stage(fragment)]]
fn main_fragment() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0);
}