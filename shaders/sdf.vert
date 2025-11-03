#version 450

// Fullscreen quad vertex shader for SDF rendering
layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out vec3 fragWorldPos;

vec2 positions[6] = vec2[](
    vec2(-1.0, -1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2( 1.0,  1.0),
    vec2(-1.0,  1.0)
);

vec2 texCoords[6] = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 1.0)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    fragTexCoord = texCoords[gl_VertexIndex];
    
    // Calculate world position for ray marching
    // This creates a view space that goes from -2 to 2 in all directions
    fragWorldPos = vec3(positions[gl_VertexIndex] * 2.0, -2.0);
}