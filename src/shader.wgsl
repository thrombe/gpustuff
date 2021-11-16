
[[block]]
struct Stuff {
    width: f32;
    height: f32;
    time: f32;
    something: f32;
};
[[group(0), binding(0)]]
var<uniform> stuff: Stuff;





fn myan(x: f32, y: f32) -> vec3<f32> {
    var time = stuff.time *10.0;
    // time = sin(time);

    var f = cos(x*x+y*y + time) - x*y/4.0;
    // var f = sin(exp(x)) - y;

    f = abs(f);
    f = pow(f, 0.3);
    f = 0.8 - f;
    let color = vec3<f32>(0.0, 1.0, 0.0);
    return vec3<f32>(f) * color;
}



[[stage(fragment)]]
fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let offset = vec2<f32>(0.0, 0.0);
    var scale = 10.0;
    var side = min(stuff.width, stuff.height); // dynamic scaling
    // var side = 300.0; // static scale



    var pos = vec2<f32>(pos.x/stuff.width, pos.y/stuff.height); // get pos from 0 to 1
    pos.y = 1.0-pos.y; // inverting y axis to get it upright
    pos = pos - vec2<f32>(0.5, 0.5); // (0, 0) at centre of screen
    pos = pos + offset;
    var pos = vec2<f32>(pos.x*stuff.width/side, pos.y*stuff.height/side);
    pos = pos * scale; // control scale

    let col = myan(pos.x, pos.y);
    return vec4<f32>(sign(col)*col*col, 1.0); // gamma correction ruines stuff
    // return vec4<f32>(stuff.time, 0.33, 0.33, 1.0);
}




[[stage(vertex)]]
fn main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 1.0);
}
