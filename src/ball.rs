use std::f32::consts::{E, PI};

use macroquad::{
    audio::{play_sound, PlaySoundParams, Sound},
    color::WHITE,
    math::{vec2, FloatExt, Vec2},
    prelude::{gl_use_default_material, gl_use_material},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

use crate::{assets::GameAssets, Settings};

const MIN_SOUND_TIME: f32 = 1.0 / 60.0;

pub struct Ball {
    position: Vec2,
    velocity: Vec2,
    rotation: f32,
    rotation_velocity: f32,
    vertical_sound_timer: f32,
    horizontal_sound_timer: f32,
    pub radius: f32,
    pub texture: Texture2D,
    pub sounds: Vec<Sound>,
}

impl Ball {
    pub fn new(texture: Texture2D, radius: f32, sounds: Vec<Sound>) -> Ball {
        Ball {
            position: Vec2::new(0., 0.),
            velocity: Vec2::ZERO,
            rotation: 0.,
            rotation_velocity: 0.,
            vertical_sound_timer: 0.,
            horizontal_sound_timer: 0.,
            radius,
            texture,
            sounds,
        }
    }

    /// Runs the physics for the ball. Returns the deltatime that is left to be simulated.
    pub fn step(
        &mut self,
        dt: f32,
        settings: &Settings,
        visual_box_velocity: Vec2,
        smoothed_box_velocity: Vec2,
        wall_hits: &mut [u8; 2],
        box_size: Vec2,
    ) -> f32 {
        let box_thickness = settings.box_thickness as f32;
        let box_depth = settings.box_depth as f32;
        let box_offset = box_thickness + box_depth;

        let temp = wall_hits[0];
        wall_hits[0] = wall_hits[1];
        wall_hits[1] = temp;

        let old_velocity = self.velocity;
        let old_position = self.position;

        let mut hit_wall_speed = vec2(0., 0.);

        let wall_and_ball_offset = self.radius + box_offset;

        // Do physics calculations

        let velocity_acceleration = Vec2::new(0., settings.gravity_strength * 1000.) * dt
            - self.velocity * (settings.air_friction * dt.clamp(0., 1.));

        self.velocity += velocity_acceleration * 0.5;
        if self.velocity.length() > settings.max_velocity * 1000. {
            self.velocity = self.velocity.normalize() * settings.max_velocity * 1000.;
        }

        self.position += (self.velocity + visual_box_velocity) * dt;

        self.velocity += velocity_acceleration * 0.5;
        if self.velocity.length() > settings.max_velocity * 1000. {
            self.velocity = self.velocity.normalize() * settings.max_velocity * 1000.;
        }

        let smoothed_total_velocity = self.velocity + smoothed_box_velocity;

        let mut back_amount = 0.0_f32;
        let mut back_vec = vec2(0., 0.);

        let distance_to_floor = box_size.y - wall_and_ball_offset - self.position.y;
        let distance_to_ceiling = self.position.y + box_size.y - wall_and_ball_offset;
        let distance_to_right_wall = box_size.x - wall_and_ball_offset - self.position.x;
        let distance_to_left_wall = self.position.x + box_size.x - wall_and_ball_offset;

        // Check how far the ball got into any walls and save the amount it needs to travel back to unintersect.
        // The highest back travel will be used, unless the ball hit a wall the previous step, and hit the same wall again.
        // That wall will not be counted.

        if distance_to_floor <= 0. {
            // Floor
            let back_for_axis = back_amount.max(
                1.0 - calculate_normalized_pos(
                    old_position.y,
                    self.position.y,
                    self.position.y + distance_to_floor,
                ),
            );
            back_vec.y = back_vec.y.max(back_for_axis);
            if !wall_hits.contains(&1) {
                back_amount = back_for_axis
            }
        }
        if distance_to_ceiling <= 0. {
            // Ceiling
            let back_for_axis = back_amount.max(
                1.0 - calculate_normalized_pos(
                    self.position.y,
                    old_position.y,
                    old_position.y + distance_to_ceiling,
                ),
            );
            back_vec.y = back_vec.y.max(back_for_axis);
            if !wall_hits.contains(&2) {
                back_amount = back_for_axis
            }
        }
        if distance_to_right_wall <= 0. {
            // Right
            let back_for_axis = back_amount.max(
                1.0 - calculate_normalized_pos(
                    old_position.x,
                    self.position.x,
                    self.position.x + distance_to_right_wall,
                ),
            );
            back_vec.x = back_vec.x.max(back_for_axis);
            if !wall_hits.contains(&3) {
                back_amount = back_for_axis
            }
        }

        if distance_to_left_wall <= 0. {
            // Left
            let back_for_axis = back_amount.max(
                1.0 - calculate_normalized_pos(
                    self.position.x,
                    old_position.x,
                    old_position.x + distance_to_left_wall,
                ),
            );
            back_vec.x = back_vec.x.max(back_for_axis);
            if !wall_hits.contains(&4) {
                back_amount = back_for_axis
            }
        }

        // Move the ball back and get new delta time.

        let new_dt = dt * (1.0 - back_amount);

        back_vec = back_vec.max(vec2(back_amount, back_amount));

        self.position = vec2(
            self.position.x.lerp(old_position.x, back_vec.x),
            self.position.y.lerp(old_position.y, back_vec.y),
        );
        self.velocity = vec2(
            self.velocity.x.lerp(old_velocity.x, back_vec.x),
            self.velocity.y.lerp(old_velocity.y, back_vec.y),
        );

        self.rotation += self.rotation_velocity * new_dt;
        self.rotation %= PI * 2.;

        let distance_to_floor = box_size.y - wall_and_ball_offset - self.position.y;
        let distance_to_ceiling = self.position.y + box_size.y - wall_and_ball_offset;
        let distance_to_right_wall = box_size.x - wall_and_ball_offset - self.position.x;
        let distance_to_left_wall = self.position.x + box_size.x - wall_and_ball_offset;

        // Putting this to 0 seems to work fine. But just in case, I will put a small number above 0.
        const SMALL_NUMBER: f32 = 0.0001;

        let mut new_last_hit_wall = wall_hits[0];

        // Calculate and apply wall interactions.
        // If it hit the wall the previous step, it will not calculate bounce, but still calculate spin.
        // (I don't remember why I did that but it's probably for a reason.)

        if distance_to_floor <= SMALL_NUMBER {
            // Floor
            hit_wall_speed.y = hit_wall_speed.y.max(smoothed_total_velocity.y.abs());
            self.position.y = box_size.y - wall_and_ball_offset;

            if !wall_hits.contains(&1) {
                new_last_hit_wall = 1;
                self.velocity.y = self
                    .velocity
                    .y
                    .min(-self.velocity.y * settings.ball_bounciness - smoothed_box_velocity.y);
            }

            (self.rotation_velocity, self.velocity.x) = calculate_bounce_spin(
                self.velocity.x,
                visual_box_velocity.x,
                self.rotation_velocity,
                self.radius,
                settings.ball_weight,
                settings.ball_friction,
                false,
            );
        }
        if distance_to_ceiling <= SMALL_NUMBER {
            // Ceiling
            hit_wall_speed.y = hit_wall_speed.y.max(smoothed_total_velocity.y.abs());
            self.position.y = -box_size.y + wall_and_ball_offset;

            if !wall_hits.contains(&2) {
                new_last_hit_wall = 2;
                self.velocity.y = self
                    .velocity
                    .y
                    .max(-self.velocity.y * settings.ball_bounciness - smoothed_box_velocity.y);
            }

            (self.rotation_velocity, self.velocity.x) = calculate_bounce_spin(
                self.velocity.x,
                visual_box_velocity.x,
                self.rotation_velocity,
                self.radius,
                settings.ball_weight,
                settings.ball_friction,
                true,
            );
        }
        if distance_to_right_wall <= SMALL_NUMBER {
            // Right
            hit_wall_speed.x = hit_wall_speed.x.max(smoothed_total_velocity.x.abs());
            self.position.x = box_size.x - wall_and_ball_offset;

            if !wall_hits.contains(&3) {
                new_last_hit_wall = 3;
                self.velocity.x = self
                    .velocity
                    .x
                    .min(-self.velocity.x * settings.ball_bounciness - smoothed_box_velocity.x);
            }

            (self.rotation_velocity, self.velocity.y) = calculate_bounce_spin(
                self.velocity.y,
                visual_box_velocity.y,
                self.rotation_velocity,
                self.radius,
                settings.ball_weight,
                settings.ball_friction,
                true,
            );
        }

        if distance_to_left_wall <= SMALL_NUMBER {
            // Left
            hit_wall_speed.x = hit_wall_speed.x.max(smoothed_total_velocity.x.abs());
            self.position.x = -box_size.x + wall_and_ball_offset;

            if !wall_hits.contains(&4) {
                new_last_hit_wall = 4;
                self.velocity.x = self
                    .velocity
                    .x
                    .max(-self.velocity.x * settings.ball_bounciness - smoothed_box_velocity.x);
            }

            (self.rotation_velocity, self.velocity.y) = calculate_bounce_spin(
                self.velocity.y,
                visual_box_velocity.y,
                self.rotation_velocity,
                self.radius,
                settings.ball_weight,
                settings.ball_friction,
                false,
            );
        }

        wall_hits[0] = new_last_hit_wall;

        let density = settings.hit_density;
        let speed_limit = settings.min_hit_speed;

        let horizontal_sound = self.horizontal_sound_timer <= 0.;
        let vertical_sound = self.vertical_sound_timer <= 0.;

        // Play sound

        if ((horizontal_sound && hit_wall_speed.x > speed_limit)
            || (vertical_sound && hit_wall_speed.y > speed_limit))
            && !self.sounds.is_empty()
        {
            let inverted_distances_from_corners =
                self.position.abs() + vec2(0., box_size.x - box_size.y);

            let mut sound_volume = hit_wall_speed.max_element();

            // The closer to the center it is, the louder the sound.
            let distance_from_corner = box_size.x - inverted_distances_from_corners.min_element();
            sound_volume -= speed_limit;
            sound_volume /= 450.;
            sound_volume *= 1. + distance_from_corner / 200.;
            let volume = 1. - 1. / E.powf(sound_volume * sound_volume * density * density);
            play_sound(
                &self.sounds[quad_rand::gen_range(0, self.sounds.len())],
                PlaySoundParams {
                    looped: false,
                    volume: volume * settings.audio_volume,
                },
            );
        }

        self.horizontal_sound_timer -= new_dt;
        self.vertical_sound_timer -= new_dt;

        if hit_wall_speed.x != 0. {
            self.horizontal_sound_timer = MIN_SOUND_TIME;
        }
        if hit_wall_speed.y != 0. {
            self.vertical_sound_timer = MIN_SOUND_TIME;
        }

        return dt - new_dt;
    }

    pub fn render(&mut self, game_assets: &GameAssets, settings: &Settings, box_size: Vec2) {
        let box_thickness = settings.box_thickness as f32;
        let box_depth = settings.box_depth as f32;
        let box_offset = box_thickness + box_depth;
        let wall_and_ball_offset = self.radius + box_offset;

        let distance_to_floor = box_size.y - wall_and_ball_offset - self.position.y;
        let distance_to_ceiling = self.position.y + box_size.y - wall_and_ball_offset;
        let distance_to_right_wall = box_size.x - wall_and_ball_offset - self.position.x;
        let distance_to_left_wall = self.position.x + box_size.x - wall_and_ball_offset;

        gl_use_material(&game_assets.shadow_material);

        // Draw shadows on box

        game_assets
            .shadow_material
            .set_uniform("shadow_strength", settings.shadow_strength);

        game_assets.shadow_material.set_uniform(
            "in_shadow",
            distance_to_floor / self.radius / settings.shadow_distance_strength,
        );

        draw_texture_ex(
            &game_assets.missing_texture,
            self.position.x - self.radius * settings.shadow_size,
            box_size.y - box_offset - box_depth,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    self.radius * settings.shadow_size * 2.,
                    box_depth * 2.,
                )),
                ..Default::default()
            },
        );

        game_assets.shadow_material.set_uniform(
            "in_shadow",
            distance_to_ceiling / self.radius / settings.shadow_distance_strength,
        );

        draw_texture_ex(
            &game_assets.missing_texture,
            self.position.x - self.radius * settings.shadow_size,
            -box_size.y + box_thickness,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    self.radius * settings.shadow_size * 2.,
                    box_depth * 2.,
                )),
                ..Default::default()
            },
        );

        game_assets.shadow_material.set_uniform(
            "in_shadow",
            distance_to_right_wall / self.radius / settings.shadow_distance_strength,
        );

        draw_texture_ex(
            &game_assets.missing_texture,
            box_size.x - box_offset - box_depth,
            self.position.y - self.radius * settings.shadow_size,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    box_depth * 2.,
                    self.radius * settings.shadow_size * 2.,
                )),
                ..Default::default()
            },
        );

        game_assets.shadow_material.set_uniform(
            "in_shadow",
            distance_to_left_wall / self.radius / settings.shadow_distance_strength,
        );

        draw_texture_ex(
            &game_assets.missing_texture,
            -box_size.x + box_thickness,
            self.position.y - self.radius * settings.shadow_size,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    box_depth * 2.,
                    self.radius * settings.shadow_size * 2.,
                )),
                ..Default::default()
            },
        );

        // Draw ball

        gl_use_material(&game_assets.ball_material);

        game_assets
            .ball_material
            .set_uniform("rotation", self.rotation);
        game_assets.ball_material.set_uniform(
            "floor_distance",
            distance_to_floor / self.radius / settings.shadow_distance_strength,
        );
        game_assets.ball_material.set_uniform(
            "ceil_distance",
            distance_to_ceiling / self.radius / settings.shadow_distance_strength,
        );
        game_assets.ball_material.set_uniform(
            "left_distance",
            distance_to_left_wall / self.radius / settings.shadow_distance_strength,
        );
        game_assets.ball_material.set_uniform(
            "right_distance",
            distance_to_right_wall / self.radius / settings.shadow_distance_strength,
        );
        game_assets
            .ball_material
            .set_uniform("ball_radius", self.radius);
        game_assets
            .ball_material
            .set_uniform("ambient_occlusion_focus", settings.ambient_occlusion_focus);
        game_assets.ball_material.set_uniform(
            "ambient_occlusion_strength",
            settings.ambient_occlusion_strength,
        );
        game_assets
            .ball_material
            .set_uniform("ambient_light", settings.ambient_light);
        game_assets
            .ball_material
            .set_uniform("specular_focus", settings.specular_focus);
        game_assets
            .ball_material
            .set_uniform("specular_strength", settings.specular_strength);

        draw_texture_ex(
            &self.texture,
            self.position.x - self.radius,
            self.position.y - self.radius,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.radius * 2., self.radius * 2.)),
                rotation: self.rotation,
                ..Default::default()
            },
        );

        gl_use_default_material();
    }
}

fn calculate_normalized_pos(min: f32, max: f32, value: f32) -> f32 {
    if min == max {
        return 0.0;
    }

    (value - min) / (max - min)
}

pub fn calculate_bounce_spin(
    ball_velocity: f32,
    window_velocity: f32,
    ball_rotation_velocity: f32,
    mut ball_radius: f32,
    weight: f32,
    friction: f32,
    inverted: bool,
) -> (f32, f32) {
    ball_radius = ball_radius.max(0.001);

    let total_velocity = if inverted {
        -(ball_velocity + window_velocity)
    } else {
        ball_velocity + window_velocity
    };
    let rotation_velocity_from_velocity = total_velocity / ball_radius;
    let middle_rotation_velocity =
        rotation_velocity_from_velocity.lerp(ball_rotation_velocity, weight * friction);
    let current_rotation_direction_velocity = if inverted {
        -middle_rotation_velocity * ball_radius
    } else {
        middle_rotation_velocity * ball_radius
    };
    let bounce_back_rotation_velocity =
        ball_rotation_velocity.lerp(rotation_velocity_from_velocity, friction);
    return (
        bounce_back_rotation_velocity,
        current_rotation_direction_velocity - window_velocity,
    );
}
