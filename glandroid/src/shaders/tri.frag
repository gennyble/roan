#version 300 es
precision mediump float;

in vec2 VertCoord;
out vec4 FragColor;

void main() {
    FragColor = vec4(VertCoord, 0.5f, 1.0f);
} 