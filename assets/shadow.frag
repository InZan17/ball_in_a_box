#version 120
varying lowp vec2 uv;
varying lowp vec4 color;

uniform float in_shadow;

uniform sampler2D Texture;

float easing_function(float x) {
    return -(cos(3.14159265 * x) - 1.0) / 2.0;
}

void main() {
    vec2 minus_one_to_one_uv = uv * 2.0 - 1.0;

    float darkness = 1.0 - length(minus_one_to_one_uv) - in_shadow / 3.;
    gl_FragColor = vec4(0.,0., 0., easing_function(clamp(darkness, 0.0, 1.0)) / (2. + in_shadow * 2.));
}