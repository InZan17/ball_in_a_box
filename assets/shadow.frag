#version 120
varying lowp vec2 uv;
varying lowp vec4 color;

uniform float in_shadow;

uniform sampler2D Texture;

void main() {
    vec2 minus_one_to_one_uv = uv * 2.0 - 1.0;

    float darkness = (length(minus_one_to_one_uv) - in_shadow);
    gl_FragColor = vec4(darkness,darkness,darkness, 1.0);
}