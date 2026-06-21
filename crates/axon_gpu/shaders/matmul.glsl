#version 450
layout(local_size_x = 8, local_size_y = 8) in;
layout(set=0, binding=0) readonly buffer BufA  { float a[]; };
layout(set=0, binding=1) readonly buffer BufB  { float b[]; };
layout(set=0, binding=2) buffer BufO { float o[]; };
layout(push_constant) uniform PC { uint rows; uint cols; uint inner; } pc;
void main() {
    uint row = gl_GlobalInvocationID.x;
    uint col = gl_GlobalInvocationID.y;
    if (row >= pc.rows || col >= pc.cols) return;
    float sum = 0.0;
    for (uint k = 0; k < pc.inner; k++)
        sum += a[row * pc.inner + k] * b[k * pc.cols + col];
    o[row * pc.cols + col] = sum;
}
