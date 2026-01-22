use macroquad::prelude::*;
use macroquad::ui::*;

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

        root_ui().label(None, "hello megaui");
        if root_ui().button(None, "Push me") {
            println!("pushed");
        }

        next_frame().await
    }
}