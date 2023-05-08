#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tex_coords;
out vec2 v_tex_coords;

uniform mat4 pvm;
uniform float z;

void main()
{
    gl_Position = pvm * vec4(pos, z, 1.0);
    v_tex_coords = tex_coords;
}