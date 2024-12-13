#version 120
varying lowp vec2 uv;
varying lowp vec4 color;

uniform float rotation;
uniform float floor_distance;
uniform float ceil_distance;
uniform float left_distance;
uniform float right_distance;
uniform float ball_radius;
uniform float ambient_occlusion_focus;
uniform float ambient_occlusion_strength;
uniform float ambient_light;
uniform float specular_focus;
uniform float specular_strength;

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

    float antialiasing_alpha_mul = 1.0 - max((center_length + delta_uv * 2.0) - 1.0, 0.0) / (delta_uv * 2.0);

    vec2 rotated_uv = rotate(minus_one_to_one_uv, rotation);

    float z = sqrt(1.0 - minus_one_to_one_uv.x * minus_one_to_one_uv.x - minus_one_to_one_uv.y * minus_one_to_one_uv.y);

    vec3 normal = vec3(rotated_uv, z);
    
    vec3 light_dir = vec3(0.6, 0.8, -1.2);
    vec3 normalized_light_dir = normalize(light_dir);

    vec4 cardboard_shadow_color = vec4(40, 20, 8, 255) / 255.;
    vec4 ambient_color = vec4(185, 159, 123, 255) / 255.;


    float diffuse = max(dot(normal, -normalized_light_dir), 0);
    float ambient_color_influence = max(ambient_light - diffuse, 0);
    diffuse = min(diffuse + ambient_light, 1);
    float specular = pow(max(dot(normal, -normalized_light_dir), 0), specular_focus) * specular_strength;
    

    vec3 up = vec3(rotate(vec2(0,-1), 0),0);
    vec3 down = vec3(rotate(vec2(0,1), 0),0);
    vec3 left = vec3(rotate(vec2(-1,0), 0),0);
    vec3 right = vec3(rotate(vec2(1,0), 0),0);

    float ceil_strength = 1 - min(ceil_distance / 2, 1);
    float floor_strength = 1 - min(floor_distance / 2, 1);
    float left_strength = 1 - min(left_distance / 2, 1);
    float right_strength = 1 - min(right_distance / 2, 1);

    float ceil_shadow = pow(max(dot(normal * ceil_strength, up), 0), ambient_occlusion_focus) * ambient_occlusion_strength;
    float floor_shadow = pow(max(dot(normal * floor_strength, down), 0), ambient_occlusion_focus) * ambient_occlusion_strength;
    float left_shadow = pow(max(dot(normal * left_strength, left), 0), ambient_occlusion_focus) * ambient_occlusion_strength;
    float right_shadow = pow(max(dot(normal * right_strength, right), 0), ambient_occlusion_focus) * ambient_occlusion_strength;

    float total_shadow = clamp(ceil_shadow + floor_shadow + left_shadow + right_shadow, 0, ambient_occlusion_strength);
    
    vec4 texture_color = texture2D(Texture, uv) * color;

    texture_color.rgb = vec3(1.0) - texture_color.rgb;
    
    float ambient_influence = ambient_color_influence * (1 - total_shadow);

    vec4 final_color = texture_color * diffuse * (1.0 - total_shadow) + texture_color * cardboard_shadow_color * total_shadow + texture_color * ambient_influence * total_shadow + vec4(1, 1, 1, 1) * specular;

    final_color.a = texture_color.a * color.a * 1.0 + specular;

    final_color = clamp(final_color, 0.0, 1.0);

    final_color.a *= antialiasing_alpha_mul;

    gl_FragColor = final_color;
}