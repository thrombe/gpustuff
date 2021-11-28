

[[block]]
struct Stuff {
    width: f32;
    height: f32;
    time: f32;
    cursor_x: f32;
    cursor_y: f32;
};
[[group(0), binding(0)]]
var<uniform> stuff: Stuff;


let PI = 3.14159265359;
type v2f = vec2<f32>;
type v3f = vec3<f32>;
type v4f = vec4<f32>;

fn rgb2hsb(rgb: vec3<f32>) -> vec3<f32> {
    let k = vec4<f32>(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    let p = mix(vec4<f32>(rgb.bg, k.wz), vec4<f32>(rgb.gb, k.xy), step(rgb.b, rgb.g));
    let q = mix(vec4<f32>(p.xyw, rgb.r), vec4<f32>(rgb.r, p.yzx), step(p.x, rgb.r));
    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)),
                d / (q.x + e),
                q.x);
}
fn hsb2rgb(hsb: vec3<f32>) -> vec3<f32> {
    var rgb = clamp(abs((hsb.x*6.0+vec3<f32>(0.0, 4.0, 2.0) % 6.0) - 3.0) - 1.0,
                     vec3<f32>(0.0),
                     vec3<f32>(1.0));
    rgb = rgb*rgb*(3.0 - 2.0*rgb);
    return hsb.z * mix(vec3<f32>(1.0), rgb, hsb.y); // ?????
}


fn plotquations(x: f32, y: f32) -> vec3<f32> {
    var time = stuff.time *1.0;
    // time = sin(time);

    // var f = cos(x*x+y*y + time) - x*y/4.0;
    // var f = sin(exp(x)) - y;
    let p = (sin(time*3.0))*5.2;
    var f = 1.0/sqrt((x-p)*(x-p) + y*y) 
          + 1.0/sqrt((x+p)*(x+p) + y*y)
          + 1.0/sqrt((y-p)*(y-p) + x*x)
          + 1.0/sqrt((y+p)*(y+p) + x*x);

    // f = abs(f);
    // f = fract(f);
    // f = floor(f);
    // f = pow(f, 0.13);
    f = pow(f, 5.13);
    f = pow(abs(0.8 - f), 4.0);
    f = pow(abs(0.6-f), 2.0);
    f = abs(0.4-f);
    let color = vec3<f32>(3.0, 1.0, 1.6);
    return vec3<f32>(f) * color;
}

fn square(x: f32, y: f32) -> vec3<f32> {
    let side = 4.2;
    return vec3<f32>(step(abs(x), side)*step(abs(y), 0.8*side));
    // if (abs(x) < side && abs(y) < side) {
    //     return vec3<f32>(0.0);
    // } else {
    //     return vec3<f32>(1.0);
    // }
}
fn circle(x: f32, y: f32) -> vec3<f32> {
    var f = sqrt(x*x+y*y) - abs(sin(stuff.time*1.9))*4.0 - 1.0;
    f = pow(f, 10.0);
    // col = floor(col);
    return vec3<f32>(f);
}
fn polar_function(x: f32, y: f32) -> vec3<f32> {
    var l = length(v2f(x, y));
    var theta = atan2(y, x);
    var r = 2.0 + 4.0*sin(sin(stuff.time)*20.0*theta + stuff.time*10.0);
    var f = smoothStep(0.0, 0.3, -l+r);
    return v3f(f);
}
fn regular_polygon(x: f32, y: f32) -> vec3<f32> {
    var l = length(v2f(x, y));
    var theta = atan2(y, x);
    var sides = 5.0;
    sides = sin(stuff.time)*10.0;
    var a = 2.0*PI/sides;
    var f = cos(floor(0.5 + theta/a)*a - theta)*l;
    f = f*0.1;
    f = smoothStep(0.4, 0.43, f);
    return v3f(f);
}
fn dot_at_mouse_position(x: f32, y: f32, cx: f32, cy: f32) -> vec3<f32> {
    return v3f(length(v2f(x-cx, y-cy)));
}
fn mandlebrot(x: f32, y: f32, curx: f32, cury: f32) -> v3f {
    var x = x*0.2;
    let curx = curx*0.2;
    let cury = cury*0.2;
    var y = y*0.2;
    let cx = x;
    let cy = y;

    var iter = 0.0;
    for (var i=0; i<500; i = i+1) {
        let ex = x;
        x = x*x-y*y + cx;
        y = 2.0*ex*y + cy;
        if (x*x+y*y > 4.0) {
            iter = f32(i);
            break;
        }
    }

    var col = v3f(iter/100.0)*1.0;
    // col = col + 1.0-clamp(4.0*dot_at_mouse_position(cx, cy, curx, cury), v3f(0.0), v3f(1.0));
    col = col*2.0 + 1.0 - 4.0*dot_at_mouse_position(cx, cy, curx, cury);

    // return v3f(length(v3f(x, y, 0.0)));
    return col;
}


[[stage(fragment)]]
fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let offset = vec2<f32>(0.0, 0.0);
    var scale = 15.0;
    var side = min(stuff.width, stuff.height); // dynamic scaling
    // var side = 300.0; // static scale



    var pos = vec2<f32>(pos.x/stuff.width, pos.y/stuff.height); // get pos from 0 to 1
    pos.y = 1.0-pos.y; // inverting y axis to get it upright
    pos = pos - vec2<f32>(0.5, 0.5); // (0, 0) at centre of screen
    pos = pos + offset;
    pos = vec2<f32>(pos.x*stuff.width/side, pos.y*stuff.height/side);
    pos = pos * scale; // control scale

    // transform cursor the same as pos
    var curs = v2f(stuff.cursor_x/stuff.width, 1.0-stuff.cursor_y/stuff.height) - v2f(0.5) + offset;
    curs = v2f(curs.x*stuff.width/side, curs.y*stuff.height/side)*scale;

    var col = mandlebrot(pos.x, pos.y, curs.x, curs.y);
    return vec4<f32>(sign(col)*col*col, 1.0); // gamma correction ruines stuff
    // return vec4<f32>(stuff.time, 0.33, 0.33, 1.0);
}




[[stage(vertex)]]
fn main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 1.0);
}
