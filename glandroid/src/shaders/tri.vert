#version 300 es
layout (location = 0) in vec2 inVertCoords;

out vec2 VertCoord;

void main() {
    VertCoord = inVertCoords;
    gl_Position = vec4(inVertCoords, 0.0, 1.0);
}