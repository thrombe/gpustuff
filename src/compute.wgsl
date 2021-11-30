
[[block]]
struct AAA {
    buff: array<u32>;
};
[[group(0), binding(1)]]
var<storage, read_write> compute_buffer: AAA;


[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    let index = global_invocation_id.x;
    compute_buffer.buff[index] = u32(1);
}