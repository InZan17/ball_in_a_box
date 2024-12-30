use macroquad::prelude::*;

use crate::assets::GameAssets;

pub fn render_mouse_tutorial(
    game_assets: &GameAssets,
    time: f32,
    time_of_understanding: Option<f32>,
    box_size: Vec2,
) {
    const FADE_TIME: f32 = 0.7;
    const FADE_SPEED: f32 = 1.5;
    const CLICK_TIME: f32 = 0.5;
    const CLICK_SPEED: f32 = 2.0;
    const MOVE_DURATION: f32 = 1.2;
    const CYCLE_DURATION: f32 = MOVE_DURATION + FADE_TIME * 2. + CLICK_TIME * 2.;

    const CURSOR_SIZE: f32 = 150.;
    const CURSOR_HALF_SIZE: f32 = -CURSOR_SIZE / 2.;

    const DRAW_TEXTURE_PARAMS: DrawTextureParams = DrawTextureParams {
        dest_size: Some(vec2(CURSOR_SIZE, CURSOR_SIZE)),
        source: None,
        rotation: 0.,
        flip_x: false,
        flip_y: false,
        pivot: None,
    };

    let cycle_time = time % CYCLE_DURATION;

    if let Some(time_of_understanding) = time_of_understanding {
        // Let the cycle finish before disabling the tutorial.
        if time_of_understanding < time - cycle_time {
            return;
        }
    }

    let start_y = -box_size.y / 3.;
    let end_y = box_size.y / 3.;

    if cycle_time < FADE_TIME {
        let time_since = cycle_time;
        let alpha = (time_since / FADE_TIME * FADE_SPEED).min(1.0);
        draw_texture_ex(
            &game_assets.mouse_normal,
            0.,
            start_y + CURSOR_HALF_SIZE,
            Color::new(1., 1., 1., alpha),
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.mouse_normal,
            -CURSOR_SIZE,
            start_y + CURSOR_HALF_SIZE,
            Color::new(1., 1., 1., alpha),
            DRAW_TEXTURE_PARAMS,
        );
    } else if cycle_time - FADE_TIME < CLICK_TIME {
        let time_since = cycle_time - FADE_TIME;
        if time_since * CLICK_SPEED < CLICK_TIME {
            draw_texture_ex(
                &game_assets.mouse_hold_move,
                0.,
                start_y + CURSOR_HALF_SIZE,
                WHITE,
                DRAW_TEXTURE_PARAMS,
            );
        } else {
            draw_texture_ex(
                &game_assets.mouse_normal_move,
                0.,
                start_y + CURSOR_HALF_SIZE,
                WHITE,
                DRAW_TEXTURE_PARAMS,
            );
        }
        draw_texture_ex(
            &game_assets.mouse_hold_move,
            -CURSOR_SIZE,
            start_y + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
    } else if cycle_time - FADE_TIME - CLICK_TIME < MOVE_DURATION {
        let time_since = cycle_time - FADE_TIME - CLICK_TIME;
        let move_precentage = time_since / MOVE_DURATION;
        draw_texture_ex(
            &game_assets.mouse_normal_move,
            0.,
            start_y.lerp(end_y, move_precentage) + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.mouse_hold_move,
            -CURSOR_SIZE,
            start_y.lerp(end_y, move_precentage) + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
    } else if cycle_time - FADE_TIME - CLICK_TIME - MOVE_DURATION < CLICK_TIME {
        let time_since = cycle_time - FADE_TIME - CLICK_TIME - MOVE_DURATION;
        if time_since * CLICK_SPEED < CLICK_TIME {
            draw_texture_ex(
                &game_assets.mouse_normal_move,
                0.,
                end_y + CURSOR_HALF_SIZE,
                WHITE,
                DRAW_TEXTURE_PARAMS,
            );
        } else {
            draw_texture_ex(
                &game_assets.mouse_hold_move,
                0.,
                end_y + CURSOR_HALF_SIZE,
                WHITE,
                DRAW_TEXTURE_PARAMS,
            );
        }
        draw_texture_ex(
            &game_assets.mouse_hold_move,
            -CURSOR_SIZE,
            end_y + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
    } else {
        let time_since = cycle_time - FADE_TIME - CLICK_TIME - MOVE_DURATION - CLICK_TIME;
        let alpha = ((1.0 - time_since / FADE_TIME) * FADE_SPEED).min(1.0);
        draw_texture_ex(
            &game_assets.mouse_normal,
            0.,
            end_y + CURSOR_HALF_SIZE,
            Color::new(1., 1., 1., alpha),
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.mouse_normal,
            -CURSOR_SIZE,
            end_y + CURSOR_HALF_SIZE,
            Color::new(1., 1., 1., alpha),
            DRAW_TEXTURE_PARAMS,
        );
    }
}
