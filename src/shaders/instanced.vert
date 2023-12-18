#version 140
in vec3 in_pos;
in vec4 in_color;

out lowp vec4 color;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 world[2];

void main() {
    gl_Position = perspective*view*world[gl_InstanceID]*vec4(in_pos, 1.0);
    color = in_color;
}