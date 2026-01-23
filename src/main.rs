use macroquad::prelude::*;
// use macroquad::ui::*;

/// Window configuration
// fn window_conf() -> Conf {
//     Conf {
//         window_title: "My Game".to_owned(),
//         window_width: 1920,
//         window_height: 1080,
//         // fullscreen: true,
//         ..Default::default()
//     }
// }




#[macroquad::main("rpg")]
async fn main() {
    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;

    loop {
        clear_background(DARKPURPLE);

        // PC端
        if is_key_down(KeyCode::Right) {
            x += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            x -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            y += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            y -= 1.0;
        }

        // 触屏输入（手机端）
        for touch in touches() {
            x = touch.position.x;
            y = touch.position.y;
        }
        draw_circle(x, y, 16.0, YELLOW);

        next_frame().await
    }
}