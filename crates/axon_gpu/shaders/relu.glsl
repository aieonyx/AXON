#version 450
layout(local_size_x = 64) in;
layout(set=0, binding=0) readonly buffer BufA  { float a[]; };
layout(set=0, binding=1) buffer BufO { float o[]; };
layout(push_constant) uniform PC { uint n; } pc;
void main() {
    uint i = gl_GlobalInvocationID.x;
    if (i < pc.n) o[i] = max(a[i], 0.0);
}
