#version 300 es
precision mediump float;

uniform vec4 renderColor;

out vec4 finalColor;

void main() {
    finalColor = renderColor;
}
