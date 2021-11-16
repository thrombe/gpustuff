

[[stage(vertex)]]
fn main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 1.0);
}


[[block]]
struct Stuff {
    width: u32;
    height: u32;
    time: f32;
    something: f32;
};
[[group(0), binding(0)]]
var<uniform> stuff: Stuff;

[[stage(fragment)]]
fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    // return vec4<f32>(pos/1080.0);
    return vec4<f32>(stuff.time, 0.33, 0.33, 1.0);
}