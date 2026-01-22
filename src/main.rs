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
    loop {
        clear_background(DARKPURPLE);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

        draw_text("HELLO", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}