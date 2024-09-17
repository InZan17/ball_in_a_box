#version 120
varying lowp vec2 uv;
varying lowp vec4 color;

uniform sampler2D Texture;

void main() {
    vec2 minus_one_to_one_uv = uv * 2.0 - 1.0;
    if (length(minus_one_to_one_uv) > 1.0) {
        discard;
    }
    gl_FragColor = texture2D(Texture, uv) * color;
}