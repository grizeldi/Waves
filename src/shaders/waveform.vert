#version 310 es

uniform float multiplier;
layout(std430, binding = 0) buffer ValueBuffer {
    float signalValues[];
};

layout(location = 0) in vec3 position;
layout(location = 1) in int id;

void main() {
    gl_Position = vec4(position.x, position.y * multiplier * signalValues[id], position.z, 1.0);
}