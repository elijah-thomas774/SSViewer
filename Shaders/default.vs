#version 330

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec4 aColor;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

out vec4 fColor;

void main() {
    fColor = aColor;
    gl_Position = proj * view *  model * vec4(aPos, 1.0);
}