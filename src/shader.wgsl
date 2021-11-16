

[[stage(vertex)]]
fn main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 1.0);
}


[[block]]
struct Stuff {
    width: f32;
    height: f32;
    time: f32;
    something: f32;
};
[[group(0), binding(0)]]
var<uniform> stuff: Stuff;

[[stage(fragment)]]
fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let pos = vec3<f32>(pos.x/stuff.width, pos.y/stuff.height, pos.z); // get pos from 0 to 1

    

    let col = pos; // consider this as final color
    return vec4<f32>(col*col, 1.0); // gamma correction ruines stuff
    // return vec4<f32>(col.x, col.y, 0.0, 1.0);
    // return vec4<f32>(stuff.time, 0.33, 0.33, 1.0);
    // return vec4<f32>(0.12, 0.33, 0.33, 1.0);
}