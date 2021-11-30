
/// compute_enabled

[[stage(compute), workgroup_size(64)]] // workgroup_size can take 3 arguments -> x*y*z executions (default x, 1, 1) // minimum opengl requirements are (1024, 1024, 64) but (x*y*z < 1024 (not too sure)) no info about wgsl rn
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let index = global_invocation_id.x;
    atomicStore(&compute_buffer.buff[index], u32(0));
    // compute_buffer.buff[index] = u32(0);
}

[[stage(fragment)]]
fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var col = atomicLoad(&compute_buffer.buff[u32(pos.x + pos.y*1080.0)]);
    // var col = compute_buffer.buff[u32(pos.x + pos.y*1080.0)];
    var col = v3f(f32(col));
    return vec4<f32>(sign(col)*col*col, 1.0); // gamma correction ruines stuff
}