#version 300 es
layout (location = 0) in vec2 inVertCoords;
layout (location = 1) in vec3 inColor;

out vec3 Color;

void main() {
    Color = inColor;
    gl_Position = vec4(inVertCoords, 0.0, 1.0);
}