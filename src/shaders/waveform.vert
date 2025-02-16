#version 300 es

uniform float multiplier;

layout(location = 0) in vec3 position;
//layout(location = 1) in int id;

void main() {
    gl_Position = vec4(position.x, position.y * multiplier, position.z, 1.0);
}
