#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec4 color;
out vec4 v_color;

uniform mat4 transform;

void main()
{
    gl_Position = transform * vec4(pos, 0.0, 1.0);
    v_color = color;
}