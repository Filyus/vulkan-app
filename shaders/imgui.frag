#version 450

// Input from vertex shader
layout(location = 0) in vec2 frag_texcoord;
layout(location = 1) in vec4 frag_color;

// Output color
layout(location = 0) out vec4 out_color;

// Font texture sampler
layout(set = 0, binding = 0) uniform sampler2D font_texture;

void main() {
    vec4 tex_color = texture(font_texture, frag_texcoord);
    
    // Standard ImGui font rendering: multiply vertex color by texture
    // Font textures are typically white with varying alpha for the glyphs
    out_color = frag_color * tex_color;
}