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
    prev_hit_wall_speed: f32,
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
            prev_hit_wall_speed: 0.,
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
    ) -> f32 {
        let mut hit_wall_speed: f32 = 0.;

        let wall_and_ball_offset = settings.ball_radius + WALL_OFFSET;

        let _old_velocity = self.velocity;

        self.velocity += Vec2::new(0., settings.gravity_strength * 1000. * dt);

        self.velocity *= 1. - (settings.air_friction * dt.clamp(0., 1.));

        if self.velocity.length() > settings.terminal_velocity * 1000. {
            self.velocity = self.velocity.normalize() * settings.terminal_velocity * 1000.;
        }

        let total_velocity = self.velocity + (wall_velocity / dt) * 2.;

        let smoothed_total_velocity = self.velocity + smoothed_wall_velocity;

        self.position += total_velocity * dt;

        self.rotation += self.rotation_velocity * dt;

        self.rotation %= PI * 2.;

        let distance_to_floor = settings.box_height - wall_and_ball_offset - self.position.y;
        if distance_to_floor <= 0. {
            // Floor
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.y.abs());
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

        let distance_to_ceiling = self.position.y + settings.box_height - wall_and_ball_offset;
        if distance_to_ceiling <= 0. {
            // Ceiling
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.y.abs());
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
        let distance_to_right_wall = settings.box_width - wall_and_ball_offset - self.position.x;
        if distance_to_right_wall <= 0. {
            // Right
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.x.abs());
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

        let distance_to_left_wall = self.position.x + settings.box_width - wall_and_ball_offset;
        if distance_to_left_wall <= 0. {
            // Left
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.x.abs());
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

        const DENSITY: f32 = 0.32;
        const SPEED_LIMIT: f32 = 120.;

        if hit_wall_speed > SPEED_LIMIT && self.prev_hit_wall_speed == 0. {
            let inverted_distances_from_corners =
                self.position.abs() + vec2(0., settings.box_width - settings.box_height);

            let distance_from_corner =
                settings.box_width - inverted_distances_from_corners.min_element();
            // The closer to the center it is, the louder the sound.
            hit_wall_speed -= SPEED_LIMIT;
            hit_wall_speed /= 450.;
            hit_wall_speed *= 1. + distance_from_corner / 200.;
            let volume = 1. - 1. / E.powf(hit_wall_speed * hit_wall_speed * DENSITY * DENSITY);
            play_sound(
                &self.sounds[quad_rand::gen_range(0, self.sounds.len())],
                PlaySoundParams {
                    looped: false,
                    volume: volume * settings.audio_volume,
                },
            );
        }

        self.prev_hit_wall_speed = hit_wall_speed;

        return 0.0;
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
