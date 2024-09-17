#version 100
attribute vec3 position;
attribute vec4 color0;
attribute vec2 texcoord;

varying lowp vec2 uv;
varying lowp vec2 pos;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
    color = color0 / 255.0;
}