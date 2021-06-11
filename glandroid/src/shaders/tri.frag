#version 300 es
precision mediump float;

in vec3 Color;
out vec4 FragColor;

void main() {
    FragColor = vec4(Color, 1.0f);
} 