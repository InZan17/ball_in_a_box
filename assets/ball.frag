#version 120
varying lowp vec2 uv;
varying lowp vec4 color;

uniform float rotation;

uniform sampler2D Texture;

vec2 rotate(vec2 point, float r) {
    float s = sin(r);
    float c = cos(r);
    vec2 new_point = vec2(
        point.x * c - point.y * s,
        point.x * s + point.y * c
    );
    return new_point;
}

void main() {
    vec2 minus_one_to_one_uv = uv * 2.0 - 1.0;
    if (length(minus_one_to_one_uv) > 1.0) {
        discard;
    }

    float rotation_offset = -0.65;
    float shadow_cutoff = 1.0;
    float highlight_cutoff = 2.0;

    float in_shadow = clamp((length(minus_one_to_one_uv + rotate(vec2(0.0, 1.0), -rotation + rotation_offset))-shadow_cutoff)/2., 0.0, 0.5);
    float in_highlight = clamp((length(minus_one_to_one_uv + rotate(vec2(0.0, -1.5), -rotation + rotation_offset))-highlight_cutoff)/2., 0.0, 0.5);

    gl_FragColor = texture2D(Texture, uv) * (color * (1.0 - in_shadow) * (1 + in_highlight));
}