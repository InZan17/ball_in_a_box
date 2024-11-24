#version 120
varying lowp vec2 uv;
varying lowp vec4 color;

uniform float rotation;
uniform float floor_distance;
uniform float ceil_distance;
uniform float left_distance;
uniform float right_distance;
uniform float ball_radius;

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
    float center_length = length(minus_one_to_one_uv);
    if (center_length > 1.0) {
        discard;
    }

    float delta_uv = 1.0 / ball_radius;

    float antialiasing_alpha_mul = 1.0 - max((center_length + delta_uv * 2.0) - 1.0, 0) / (delta_uv * 2.0);

    float rotation_offset = -0.65;
    float shadow_cutoff = 1.0;
    float highlight_cutoff = 2.2;

    float in_shadow = clamp((length(minus_one_to_one_uv + rotate(vec2(0.0, 1.0), -rotation + rotation_offset))-shadow_cutoff)/2., 0.0, 0.5);
    float in_highlight = clamp((length(minus_one_to_one_uv + rotate(vec2(0.0, -1.7), -rotation + rotation_offset))-highlight_cutoff)/2., 0.0, 0.5);


    float shadow_cutoff2 = 2.05;
    float shadow2_roundness = 1.8;
    
    float left_shadow = clamp((length(minus_one_to_one_uv + rotate(vec2(-shadow2_roundness, 0.0), -rotation))-shadow_cutoff2 - left_distance)/2., 0.0, 0.5);
    float right_shadow = clamp((length(minus_one_to_one_uv + rotate(vec2(shadow2_roundness, 0.0), -rotation))-shadow_cutoff2 - right_distance)/2., 0.0, 0.5);
    float up_shadow = clamp((length(minus_one_to_one_uv + rotate(vec2(0.0, -shadow2_roundness), -rotation))-shadow_cutoff2 - ceil_distance)/2., 0.0, 0.5);
    float down_shadow = clamp((length(minus_one_to_one_uv + rotate(vec2(0.0, shadow2_roundness), -rotation))-shadow_cutoff2 - floor_distance)/2., 0.0, 0.5);

    float total_shadow = clamp(left_shadow + right_shadow + up_shadow + down_shadow, 0.0, 0.5);

    vec4 cardboard_shadow_color = vec4(82, 43, 12, 255) / 255.;

    vec4 texture_color = texture2D(Texture, uv);

    vec4 final_color = texture_color * color * (1.0 - in_shadow) * (1.0 - total_shadow) + texture_color * cardboard_shadow_color * total_shadow + vec4(1, 1, 1, 1) * in_highlight;

    final_color.a = texture_color.a * color.a * 1.0 + in_highlight;

    final_color = clamp(final_color, 0.0, 1.0);

    final_color.a *= antialiasing_alpha_mul;

    gl_FragColor = final_color;
    //gl_FragColor = vec4(in_shadow, in_highlight, total_shadow, 1.0);
}