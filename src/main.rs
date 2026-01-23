// use macroquad::prelude::*;
// // use macroquad::ui::*;

// /// Window configuration
// // fn window_conf() -> Conf {
// //     Conf {
// //         window_title: "My Game".to_owned(),
// //         window_width: 1920,
// //         window_height: 1080,
// //         // fullscreen: true,
// //         ..Default::default()
// //     }
// // }




// #[macroquad::main("rpg")]
// async fn main() {
//     let mut x = screen_width() / 2.0;
//     let mut y = screen_height() / 2.0;

//     loop {
//         clear_background(DARKPURPLE);

//         // PC端
//         if is_key_down(KeyCode::Right) {
//             x += 1.0;
//         }
//         if is_key_down(KeyCode::Left) {
//             x -= 1.0;
//         }
//         if is_key_down(KeyCode::Down) {
//             y += 1.0;
//         }
//         if is_key_down(KeyCode::Up) {
//             y -= 1.0;
//         }

//         // 触屏输入（手机端）
//         for touch in touches() {
//             x = touch.position.x;
//             y = touch.position.y;
//         }
//         draw_circle(x, y, 16.0, YELLOW);

//         next_frame().await
//     }
// }

use macroquad::prelude::*;
use std::collections::HashMap;

// ========== 核心数据结构 ==========

#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Floor,
    Wall,
    Door,
    Water,
}

impl TileType {
    fn as_char(&self) -> char {
        match self {
            TileType::Floor => '.',
            TileType::Wall => '#',
            TileType::Door => '+',
            TileType::Water => '~',
        }
    }
    
    fn is_walkable(&self) -> bool {
        matches!(self, TileType::Floor | TileType::Door)
    }
}

#[derive(Clone)]
struct Item {
    name: String,
    char: char,
    item_type: ItemType,
}

#[derive(Clone)]
enum ItemType {
    Weapon { damage: i32 },
    Armor { defense: i32 },
    Consumable { heal: i32 },
    Quest,
}

#[derive(Clone)]
struct NPC {
    name: String,
    char: char,
    x: i32,
    y: i32,
    hp: i32,
    max_hp: i32,
    hostile: bool,
    dialogue: Vec<String>,
}

struct Player {
    x: i32,
    y: i32,
    hp: i32,
    max_hp: i32,
    inventory: Vec<Item>,
    stats: PlayerStats,
}

struct PlayerStats {
    strength: i32,
    perception: i32,
    endurance: i32,
    charisma: i32,
    intelligence: i32,
    agility: i32,
    luck: i32,
}

struct GameMap {
    width: i32,
    height: i32,
    tiles: Vec<Vec<TileType>>,
    items: HashMap<(i32, i32), Item>,
}

impl GameMap {
    fn new(width: i32, height: i32) -> Self {
        let mut tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        
        // 创建简单的房间和走廊
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                }
            }
        }
        
        // 添加一些内部墙壁
        for x in 10..15 {
            tiles[5][x] = TileType::Wall;
        }
        tiles[5][12] = TileType::Door;
        
        // 添加水域
        for x in 20..25 {
            for y in 8..12 {
                tiles[y][x] = TileType::Water;
            }
        }
        
        let mut items = HashMap::new();
        items.insert((5, 5), Item {
            name: "Stimpak".to_string(),
            char: '+',
            item_type: ItemType::Consumable { heal: 20 },
        });
        items.insert((15, 8), Item {
            name: "Pistol".to_string(),
            char: ')',
            item_type: ItemType::Weapon { damage: 10 },
        });
        
        GameMap { width, height, tiles, items }
    }
    
    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        self.tiles[y as usize][x as usize].is_walkable()
    }
}

enum GameState {
    Playing,
    Inventory,
    Dialogue(usize), // NPC索引
    Combat(usize),   // NPC索引
}

struct Game {
    player: Player,
    map: GameMap,
    npcs: Vec<NPC>,
    state: GameState,
    messages: Vec<String>,
    camera_x: i32,
    camera_y: i32,
}

impl Game {
    fn new() -> Self {
        let player = Player {
            x: 5,
            y: 5,
            hp: 100,
            max_hp: 100,
            inventory: vec![],
            stats: PlayerStats {
                strength: 5,
                perception: 5,
                endurance: 5,
                charisma: 5,
                intelligence: 5,
                agility: 5,
                luck: 5,
            },
        };
        
        let map = GameMap::new(40, 30);
        
        let npcs = vec![
            NPC {
                name: "Trader".to_string(),
                char: 'T',
                x: 10,
                y: 10,
                hp: 50,
                max_hp: 50,
                hostile: false,
                dialogue: vec![
                    "Hello, traveler!".to_string(),
                    "Want to trade?".to_string(),
                    "Safe travels.".to_string(),
                ],
            },
            NPC {
                name: "Raider".to_string(),
                char: 'R',
                x: 20,
                y: 15,
                hp: 60,
                max_hp: 60,
                hostile: true,
                dialogue: vec!["Die!".to_string()],
            },
        ];
        
        Game {
            player,
            map,
            npcs,
            state: GameState::Playing,
            messages: vec!["Welcome to the Wasteland!".to_string()],
            camera_x: 0,
            camera_y: 0,
        }
    }
    
    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }
    
    fn move_player(&mut self, dx: i32, dy: i32) {
        let new_x = self.player.x + dx;
        let new_y = self.player.y + dy;
        
        // 检查NPC碰撞
        if let Some(npc_idx) = self.npcs.iter().position(|n| n.x == new_x && n.y == new_y) {
            if self.npcs[npc_idx].hostile {
                self.state = GameState::Combat(npc_idx);
                self.add_message(format!("Combat with {}!", self.npcs[npc_idx].name));
            } else {
                self.state = GameState::Dialogue(npc_idx);
            }
            return;
        }
        
        // 检查地图碰撞
        if self.map.is_walkable(new_x, new_y) {
            self.player.x = new_x;
            self.player.y = new_y;
            
            // 检查物品拾取
            if let Some(item) = self.map.items.remove(&(new_x, new_y)) {
                self.add_message(format!("Picked up {}", item.name));
                self.player.inventory.push(item);
            }
        }
    }
    
    fn update_camera(&mut self) {
        self.camera_x = self.player.x - 20;
        self.camera_y = self.player.y - 10;
    }
}

// ========== 渲染系统 ==========

fn draw_game(game: &Game) {
    let tile_size = 20.0;
    let start_x = 20.0;
    let start_y = 40.0;
    
    // 绘制地图
    for y in 0..game.map.height {
        for x in 0..game.map.width {
            let screen_x = start_x + (x - game.camera_x) as f32 * tile_size;
            let screen_y = start_y + (y - game.camera_y) as f32 * tile_size;
            
            if screen_x < 0.0 || screen_y < 0.0 || screen_x > screen_width() || screen_y > screen_height() {
                continue;
            }
            
            let tile = game.map.tiles[y as usize][x as usize];
            let color = match tile {
                TileType::Floor => DARKGRAY,
                TileType::Wall => GRAY,
                TileType::Door => BROWN,
                TileType::Water => BLUE,
            };
            
            draw_rectangle(screen_x, screen_y, tile_size, tile_size, color);
            draw_text(
                &tile.as_char().to_string(),
                screen_x + 5.0,
                screen_y + 15.0,
                20.0,
                WHITE,
            );
        }
    }
    
    // 绘制物品
    for ((x, y), item) in &game.map.items {
        let screen_x = start_x + (*x - game.camera_x) as f32 * tile_size;
        let screen_y = start_y + (*y - game.camera_y) as f32 * tile_size;
        draw_text(
            &item.char.to_string(),
            screen_x + 5.0,
            screen_y + 15.0,
            20.0,
            YELLOW,
        );
    }
    
    // 绘制NPC
    for npc in &game.npcs {
        let screen_x = start_x + (npc.x - game.camera_x) as f32 * tile_size;
        let screen_y = start_y + (npc.y - game.camera_y) as f32 * tile_size;
        let color = if npc.hostile { RED } else { GREEN };
        draw_text(
            &npc.char.to_string(),
            screen_x + 5.0,
            screen_y + 15.0,
            20.0,
            color,
        );
    }
    
    // 绘制玩家
    let player_screen_x = start_x + (game.player.x - game.camera_x) as f32 * tile_size;
    let player_screen_y = start_y + (game.player.y - game.camera_y) as f32 * tile_size;
    draw_text("@", player_screen_x + 5.0, player_screen_y + 15.0, 20.0, SKYBLUE);
}

fn draw_ui(game: &Game) {
    // 状态栏
    draw_rectangle(0.0, 0.0, screen_width(), 30.0, BLACK);
    draw_text(
        &format!("HP: {}/{} | Pos: ({},{}) | Items: {}", 
                 game.player.hp, game.player.max_hp,
                 game.player.x, game.player.y,
                 game.player.inventory.len()),
        10.0, 20.0, 20.0, GREEN
    );
    
    // 消息日志
    let log_y = screen_height() - 120.0;
    draw_rectangle(0.0, log_y, screen_width(), 120.0, Color::new(0.0, 0.0, 0.0, 0.8));
    for (i, msg) in game.messages.iter().enumerate() {
        draw_text(msg, 10.0, log_y + 20.0 + i as f32 * 20.0, 18.0, LIGHTGRAY);
    }
    
    // 控制提示
    draw_text("WASD/Arrow: Move | I: Inventory | ESC: Menu", 10.0, screen_height() - 10.0, 16.0, DARKGRAY);
}

fn draw_inventory(game: &Game) {
    let panel_w = 400.0;
    let panel_h = 300.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
    
    draw_text("INVENTORY", panel_x + 10.0, panel_y + 30.0, 24.0, YELLOW);
    
    if game.player.inventory.is_empty() {
        draw_text("Empty", panel_x + 10.0, panel_y + 60.0, 20.0, GRAY);
    } else {
        for (i, item) in game.player.inventory.iter().enumerate() {
            draw_text(
                &format!("{} - {}", item.char, item.name),
                panel_x + 10.0,
                panel_y + 60.0 + i as f32 * 25.0,
                20.0,
                WHITE,
            );
        }
    }
    
    draw_text("Press I to close", panel_x + 10.0, panel_y + panel_h - 20.0, 16.0, DARKGRAY);
}

fn draw_dialogue(game: &Game, npc_idx: usize) {
    let panel_w = 500.0;
    let panel_h = 200.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = screen_height() - panel_h - 50.0;
    
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, GREEN);
    
    let npc = &game.npcs[npc_idx];
    draw_text(&npc.name, panel_x + 10.0, panel_y + 30.0, 22.0, GREEN);
    
    for (i, line) in npc.dialogue.iter().enumerate() {
        draw_text(line, panel_x + 10.0, panel_y + 60.0 + i as f32 * 25.0, 18.0, WHITE);
    }
    
    draw_text("Press SPACE to continue", panel_x + 10.0, panel_y + panel_h - 20.0, 16.0, DARKGRAY);
}

fn draw_combat(game: &Game, npc_idx: usize) {
    let panel_w = 500.0;
    let panel_h = 250.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, RED);
    
    let npc = &game.npcs[npc_idx];
    
    draw_text("COMBAT", panel_x + 10.0, panel_y + 30.0, 24.0, RED);
    draw_text(&format!("Enemy: {}", npc.name), panel_x + 10.0, panel_y + 60.0, 20.0, ORANGE);
    draw_text(&format!("Enemy HP: {}/{}", npc.hp, npc.max_hp), panel_x + 10.0, panel_y + 85.0, 18.0, WHITE);
    draw_text(&format!("Your HP: {}/{}", game.player.hp, game.player.max_hp), panel_x + 10.0, panel_y + 110.0, 18.0, WHITE);
    
    draw_text("1: Attack", panel_x + 10.0, panel_y + 150.0, 18.0, YELLOW);
    draw_text("2: Use Item", panel_x + 10.0, panel_y + 175.0, 18.0, YELLOW);
    draw_text("3: Run", panel_x + 10.0, panel_y + 200.0, 18.0, YELLOW);
}

// ========== 主循环 ==========

#[macroquad::main("Fallout-style RPG")]
async fn main() {
    let mut game = Game::new();
    
    loop {
        clear_background(BLACK);
        
        // 输入处理
        match game.state {
            GameState::Playing => {
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    game.move_player(0, -1);
                }
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    game.move_player(0, 1);
                }
                if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                    game.move_player(-1, 0);
                }
                if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                    game.move_player(1, 0);
                }
                if is_key_pressed(KeyCode::I) {
                    game.state = GameState::Inventory;
                }
            }
            GameState::Inventory => {
                if is_key_pressed(KeyCode::I) || is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                }
            }
            GameState::Dialogue(_) => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                }
            }
            GameState::Combat(npc_idx) => {
                if is_key_pressed(KeyCode::Key1) {
                    // 攻击
                    let damage = 15;
                    game.npcs[npc_idx].hp -= damage;
                    game.add_message(format!("You dealt {} damage!", damage));
                    
                    if game.npcs[npc_idx].hp <= 0 {
                        game.add_message(format!("{} defeated!", game.npcs[npc_idx].name));
                        game.npcs.remove(npc_idx);
                        game.state = GameState::Playing;
                    } else {
                        // 敌人反击
                        let enemy_damage = 10;
                        game.player.hp -= enemy_damage;
                        game.add_message(format!("Enemy dealt {} damage!", enemy_damage));
                    }
                }
                if is_key_pressed(KeyCode::Key3) {
                    game.add_message("You ran away!".to_string());
                    game.state = GameState::Playing;
                }
            }
        }
        
        // 更新摄像机
        game.update_camera();
        
        // 渲染
        draw_game(&game);
        draw_ui(&game);
        
        match game.state {
            GameState::Inventory => draw_inventory(&game),
            GameState::Dialogue(idx) => draw_dialogue(&game, idx),
            GameState::Combat(idx) => draw_combat(&game, idx),
            _ => {}
        }
        
        next_frame().await;
    }
}