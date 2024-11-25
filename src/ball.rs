use std::f32::consts::{E, PI};

use macroquad::{
    audio::{play_sound, PlaySoundParams, Sound},
    color::WHITE,
    math::{vec2, FloatExt, Vec2},
    prelude::{gl_use_default_material, gl_use_material, Material},
    shapes::draw_rectangle,
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

use crate::{Settings, WALL_DEPTH, WALL_OFFSET, WALL_THICKNESS};

pub struct Ball {
    position: Vec2,
    velocity: Vec2,
    rotation: f32,
    rotation_velocity: f32,
    vertical_sound: bool,
    horizontal_sound: bool,
    pub texture: Texture2D,
    pub ball_material: Material,
    pub shadow_material: Material,
    pub sounds: Vec<Sound>,
}

impl Ball {
    pub fn new(
        texture: Texture2D,
        ball_material: Material,
        shadow_material: Material,
        sounds: Vec<Sound>,
    ) -> Ball {
        Ball {
            position: Vec2::new(0., 0.),
            velocity: Vec2::ZERO,
            rotation: 0.,
            rotation_velocity: 0.,
            vertical_sound: true,
            horizontal_sound: true,
            texture,
            ball_material,
            shadow_material,
            sounds,
        }
    }

    /// Runs the physics for the ball. Returns the deltatime that is left to be simulated.
    pub fn step(
        &mut self,
        dt: f32,
        settings: &Settings,
        wall_velocity: Vec2,
        smoothed_wall_velocity: Vec2,
        wall_hits: &mut [u8; 2],
    ) -> f32 {
        let temp = wall_hits[0];
        wall_hits[0] = wall_hits[1];
        wall_hits[1] = temp;

        let old_velocity = self.velocity;
        let old_position = self.position;

        let mut hit_wall_speed = vec2(0., 0.);

        let wall_and_ball_offset = settings.ball_radius + WALL_OFFSET;

        self.velocity += Vec2::new(0., settings.gravity_strength * 1000. * dt);

        self.velocity *= 1. - (settings.air_friction * dt.clamp(0., 1.));

        if self.velocity.length() > settings.terminal_velocity * 1000. {
            self.velocity = self.velocity.normalize() * settings.terminal_velocity * 1000.;
        }

        let total_velocity = self.velocity + wall_velocity * 2.;

        let smoothed_total_velocity = self.velocity + smoothed_wall_velocity;

        self.position += total_velocity * dt;

        let mut back_amount = 0.0_f32;
        let mut back_vec = vec2(0., 0.);

        let distance_to_floor = settings.box_height - wall_and_ball_offset - self.position.y;
        let distance_to_ceiling = self.position.y + settings.box_height - wall_and_ball_offset;
        let distance_to_right_wall = settings.box_width - wall_and_ball_offset - self.position.x;
        let distance_to_left_wall = self.position.x + settings.box_width - wall_and_ball_offset;

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

        let distance_to_floor = settings.box_height - wall_and_ball_offset - self.position.y;
        let distance_to_ceiling = self.position.y + settings.box_height - wall_and_ball_offset;
        let distance_to_right_wall = settings.box_width - wall_and_ball_offset - self.position.x;
        let distance_to_left_wall = self.position.x + settings.box_width - wall_and_ball_offset;

        // Putting this to 0 seems to work fine. But just in case, I will put a small number abocve 0.
        const SMALL_NUMBER: f32 = 0.01;

        let mut new_last_hit_wall = wall_hits[0];

        if distance_to_floor <= SMALL_NUMBER {
            // Floor
            if !wall_hits.contains(&1) {
                new_last_hit_wall = 1;
            }

            hit_wall_speed.y = hit_wall_speed.y.max(smoothed_total_velocity.y.abs());
            self.position.y = settings.box_height - wall_and_ball_offset;
            self.velocity.y =
                -self.velocity.y * settings.ball_bounciness - smoothed_wall_velocity.y;

            (self.rotation_velocity, self.velocity.x) = calculate_bounce_spin(
                self.velocity.x,
                smoothed_wall_velocity.x,
                self.rotation_velocity,
                settings.ball_radius,
                settings.ball_weight,
                settings.ball_friction,
                false,
            );
        }
        if distance_to_ceiling <= SMALL_NUMBER {
            // Ceiling
            if !wall_hits.contains(&2) {
                new_last_hit_wall = 2;
            }

            hit_wall_speed.y = hit_wall_speed.y.max(smoothed_total_velocity.y.abs());
            self.position.y = -settings.box_height + wall_and_ball_offset;
            self.velocity.y =
                -self.velocity.y * settings.ball_bounciness - smoothed_wall_velocity.y;

            (self.rotation_velocity, self.velocity.x) = calculate_bounce_spin(
                self.velocity.x,
                smoothed_wall_velocity.x,
                self.rotation_velocity,
                settings.ball_radius,
                settings.ball_weight,
                settings.ball_friction,
                true,
            );
        }
        if distance_to_right_wall <= SMALL_NUMBER {
            // Right
            if !wall_hits.contains(&3) {
                new_last_hit_wall = 3;
            }

            hit_wall_speed.x = hit_wall_speed.x.max(smoothed_total_velocity.x.abs());
            self.position.x = settings.box_width - wall_and_ball_offset;
            self.velocity.x =
                -self.velocity.x * settings.ball_bounciness - smoothed_wall_velocity.x;

            (self.rotation_velocity, self.velocity.y) = calculate_bounce_spin(
                self.velocity.y,
                smoothed_wall_velocity.y,
                self.rotation_velocity,
                settings.ball_radius,
                settings.ball_weight,
                settings.ball_friction,
                true,
            );
        }

        if distance_to_left_wall <= SMALL_NUMBER {
            // Left
            if !wall_hits.contains(&4) {
                new_last_hit_wall = 4;
            }

            hit_wall_speed.x = hit_wall_speed.x.max(smoothed_total_velocity.x.abs());
            self.position.x = -settings.box_width + wall_and_ball_offset;
            self.velocity.x =
                -self.velocity.x * settings.ball_bounciness - smoothed_wall_velocity.x;

            (self.rotation_velocity, self.velocity.y) = calculate_bounce_spin(
                self.velocity.y,
                smoothed_wall_velocity.y,
                self.rotation_velocity,
                settings.ball_radius,
                settings.ball_weight,
                settings.ball_friction,
                false,
            );
        }

        wall_hits[0] = new_last_hit_wall;

        const DENSITY: f32 = 0.32;
        const SPEED_LIMIT: f32 = 120.;

        if (self.horizontal_sound && hit_wall_speed.x > SPEED_LIMIT)
            || (self.vertical_sound && hit_wall_speed.y > SPEED_LIMIT)
        {
            let inverted_distances_from_corners =
                self.position.abs() + vec2(0., settings.box_width - settings.box_height);

            let mut sound_volume = hit_wall_speed.max_element();

            // The closer to the center it is, the louder the sound.
            let distance_from_corner =
                settings.box_width - inverted_distances_from_corners.min_element();
            sound_volume -= SPEED_LIMIT;
            sound_volume /= 450.;
            sound_volume *= 1. + distance_from_corner / 200.;
            let volume = 1. - 1. / E.powf(sound_volume * sound_volume * DENSITY * DENSITY);
            play_sound(
                &self.sounds[quad_rand::gen_range(0, self.sounds.len())],
                PlaySoundParams {
                    looped: false,
                    volume: volume * settings.audio_volume,
                },
            );
        }

        self.horizontal_sound = hit_wall_speed.x == 0.0;
        self.vertical_sound = hit_wall_speed.y == 0.0;

        return dt - new_dt;
    }

    pub fn render(&mut self, settings: &Settings) {
        let wall_and_ball_offset = settings.ball_radius + WALL_OFFSET;

        let distance_to_floor = settings.box_height - wall_and_ball_offset - self.position.y;
        let distance_to_ceiling = self.position.y + settings.box_height - wall_and_ball_offset;
        let distance_to_right_wall = settings.box_width - wall_and_ball_offset - self.position.x;
        let distance_to_left_wall = self.position.x + settings.box_width - wall_and_ball_offset;

        gl_use_material(&self.shadow_material);

        self.shadow_material.set_uniform(
            "in_shadow",
            distance_to_floor / settings.ball_radius / settings.shadow_distance_strength,
        );
        draw_rectangle(
            self.position.x - settings.ball_radius * settings.shadow_size,
            settings.box_height - WALL_OFFSET - WALL_DEPTH,
            settings.ball_radius * settings.shadow_size * 2.,
            WALL_DEPTH * 2.,
            WHITE,
        );

        self.shadow_material.set_uniform(
            "in_shadow",
            distance_to_ceiling / settings.ball_radius / settings.shadow_distance_strength,
        );
        draw_rectangle(
            self.position.x - settings.ball_radius * settings.shadow_size,
            -settings.box_height + WALL_THICKNESS,
            settings.ball_radius * settings.shadow_size * 2.,
            WALL_DEPTH * 2.,
            WHITE,
        );

        self.shadow_material.set_uniform(
            "in_shadow",
            distance_to_right_wall / settings.ball_radius / settings.shadow_distance_strength,
        );
        draw_rectangle(
            settings.box_width - WALL_OFFSET - WALL_DEPTH,
            self.position.y - settings.ball_radius * settings.shadow_size,
            WALL_DEPTH * 2.,
            settings.ball_radius * settings.shadow_size * 2.,
            WHITE,
        );

        self.shadow_material.set_uniform(
            "in_shadow",
            distance_to_left_wall / settings.ball_radius / settings.shadow_distance_strength,
        );
        draw_rectangle(
            -settings.box_width + WALL_THICKNESS,
            self.position.y - settings.ball_radius * settings.shadow_size,
            WALL_DEPTH * 2.,
            settings.ball_radius * settings.shadow_size * 2.,
            WHITE,
        );

        gl_use_material(&self.ball_material);

        self.ball_material.set_uniform("rotation", self.rotation);
        self.ball_material.set_uniform(
            "floor_distance",
            distance_to_floor / settings.ball_radius / settings.shadow_distance_strength,
        );
        self.ball_material.set_uniform(
            "ceil_distance",
            distance_to_ceiling / settings.ball_radius / settings.shadow_distance_strength,
        );
        self.ball_material.set_uniform(
            "left_distance",
            distance_to_left_wall / settings.ball_radius / settings.shadow_distance_strength,
        );
        self.ball_material.set_uniform(
            "right_distance",
            distance_to_right_wall / settings.ball_radius / settings.shadow_distance_strength,
        );
        self.ball_material
            .set_uniform("ball_radius", settings.ball_radius);

        draw_texture_ex(
            &self.texture,
            self.position.x - settings.ball_radius,
            self.position.y - settings.ball_radius,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(settings.ball_radius * 2., settings.ball_radius * 2.)),
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
