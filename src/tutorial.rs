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

    const SPACING: f32 = 5.;

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

    let y_pos;
    let alpha;
    let left_texture;
    let right_texture;

    if cycle_time < FADE_TIME {
        let time_since = cycle_time;
        alpha = (time_since / FADE_TIME * FADE_SPEED).min(1.0);
        y_pos = start_y;
        right_texture = &game_assets.mouse_normal;
        left_texture = &game_assets.mouse_normal;
    } else if cycle_time - FADE_TIME < CLICK_TIME {
        let time_since = cycle_time - FADE_TIME;
        alpha = 1.0;
        y_pos = start_y;
        if time_since * CLICK_SPEED < CLICK_TIME {
            right_texture = &game_assets.mouse_hold_move;
        } else {
            right_texture = &game_assets.mouse_normal_move;
        }
        left_texture = &game_assets.mouse_hold_move;
    } else if cycle_time - FADE_TIME - CLICK_TIME < MOVE_DURATION {
        let time_since = cycle_time - FADE_TIME - CLICK_TIME;
        let move_precentage = time_since / MOVE_DURATION;
        alpha = 1.0;
        y_pos = start_y.lerp(end_y, move_precentage);
        right_texture = &game_assets.mouse_normal_move;
        left_texture = &game_assets.mouse_hold_move;
    } else if cycle_time - FADE_TIME - CLICK_TIME - MOVE_DURATION < CLICK_TIME {
        let time_since = cycle_time - FADE_TIME - CLICK_TIME - MOVE_DURATION;
        alpha = 1.0;
        y_pos = end_y;
        if time_since * CLICK_SPEED < CLICK_TIME {
            right_texture = &game_assets.mouse_normal_move;
        } else {
            right_texture = &game_assets.mouse_hold_move;
        }
        left_texture = &game_assets.mouse_hold_move;
    } else {
        let time_since = cycle_time - FADE_TIME - CLICK_TIME - MOVE_DURATION - CLICK_TIME;
        alpha = ((1.0 - time_since / FADE_TIME) * FADE_SPEED).min(1.0);
        y_pos = end_y;
        right_texture = &game_assets.mouse_normal;
        left_texture = &game_assets.mouse_normal;
    }

    draw_texture_ex(
        right_texture,
        SPACING,
        y_pos + CURSOR_HALF_SIZE,
        Color::new(1., 1., 1., alpha),
        DRAW_TEXTURE_PARAMS,
    );
    draw_texture_ex(
        left_texture,
        -CURSOR_SIZE - SPACING,
        y_pos + CURSOR_HALF_SIZE,
        Color::new(1., 1., 1., alpha),
        DRAW_TEXTURE_PARAMS,
    );
    draw_texture_ex(
        &game_assets.slash,
        -CURSOR_SIZE / 2.,
        y_pos + CURSOR_HALF_SIZE,
        Color::new(1., 1., 1., alpha),
        DRAW_TEXTURE_PARAMS,
    );
}

pub fn render_menu_tutorial(game_assets: &GameAssets, time: f32) {
    const CLICK_TIME: f32 = 0.175;
    const WAIT_TIME: f32 = 1.25;
    const FADE_IN_SPEED: f32 = 2.0;
    const CYCLE_DURATION: f32 = WAIT_TIME + CLICK_TIME * 3.;

    const CURSOR_SIZE: f32 = 150.;
    const CURSOR_HALF_SIZE: f32 = -CURSOR_SIZE / 2.;
    const ESC_DOWN_OFFSET: f32 = 10.;

    const ESC_LEFT_OFFSET: f32 = 15.;

    const SPACING: f32 = 5.;

    const DRAW_TEXTURE_PARAMS: DrawTextureParams = DrawTextureParams {
        dest_size: Some(vec2(CURSOR_SIZE, CURSOR_SIZE)),
        source: None,
        rotation: 0.,
        flip_x: false,
        flip_y: false,
        pivot: None,
    };

    let alpha = (time * FADE_IN_SPEED).min(1.);

    let cycle_time = time % CYCLE_DURATION;

    if cycle_time < WAIT_TIME {
        draw_texture_ex(
            &game_assets.mouse_normal,
            SPACING,
            CURSOR_HALF_SIZE,
            Color::new(1., 1., 1., alpha),
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.esc_normal,
            -CURSOR_SIZE - SPACING - ESC_LEFT_OFFSET,
            CURSOR_HALF_SIZE,
            Color::new(1., 1., 1., alpha),
            DRAW_TEXTURE_PARAMS,
        );
    } else if cycle_time < WAIT_TIME + CLICK_TIME {
        draw_texture_ex(
            &game_assets.mouse_hold,
            SPACING,
            CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.esc_hold,
            -CURSOR_SIZE - SPACING - ESC_LEFT_OFFSET,
            ESC_DOWN_OFFSET + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
    } else if cycle_time < WAIT_TIME + CLICK_TIME * 2. {
        draw_texture_ex(
            &game_assets.mouse_normal,
            SPACING,
            CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.esc_hold,
            -CURSOR_SIZE - SPACING - ESC_LEFT_OFFSET,
            ESC_DOWN_OFFSET + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
    } else if cycle_time < WAIT_TIME + CLICK_TIME * 3. {
        draw_texture_ex(
            &game_assets.mouse_hold,
            SPACING,
            CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
        draw_texture_ex(
            &game_assets.esc_hold,
            -CURSOR_SIZE - SPACING - ESC_LEFT_OFFSET,
            ESC_DOWN_OFFSET + CURSOR_HALF_SIZE,
            WHITE,
            DRAW_TEXTURE_PARAMS,
        );
    }

    draw_texture_ex(
        &game_assets.slash,
        -CURSOR_SIZE / 2.,
        CURSOR_HALF_SIZE,
        Color::new(1., 1., 1., alpha),
        DRAW_TEXTURE_PARAMS,
    );
}
