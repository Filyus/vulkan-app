#version 450

// Vertex input
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_texcoord;
layout(location = 2) in vec4 in_color;

// Output to fragment shader
layout(location = 0) out vec2 frag_texcoord;
layout(location = 1) out vec4 frag_color;

// Push constants for projection matrix
layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
} pc;

void main() {
    frag_texcoord = in_texcoord;
    frag_color = in_color;
    gl_Position = pc.projection_matrix * vec4(in_position, 0.0, 1.0);
}