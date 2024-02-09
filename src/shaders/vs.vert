#version 460

layout(set = 0, location = 0) in vec3 a_position;

void main() {
    gl_Position = vec4(a_position, 1.0);
}
