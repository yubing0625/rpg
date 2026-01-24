
// 导入 macroquad 游戏引擎的核心功能
// 包含图形渲染、输入处理、颜色定义等
use macroquad::prelude::*;

// 导入 HashMap 用于存储地图上的物品位置
use std::collections::HashMap;

// ========== 核心数据结构 ==========

/// 地砖类型枚举
/// 定义游戏世界中所有可能的地形类型
#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Floor,     // 地板 - 可通行
    Wall,      // 墙壁 - 不可通行
    Door,      // 门 - 可通行
    Water,     // 水域 - 不可通行
    Grass,     // 草地 - 可通行（大地图）
    Mountain,  // 山脉 - 不可通行（大地图）
    Forest,    // 森林 - 可通行（大地图）
    Town,      // 城镇入口 - 可进入
    Dungeon,   // 地牢入口 - 可进入
}

/// 地图类型枚举
/// 区分大地图和小地图（城镇/地牢）
#[derive(Clone, Copy, PartialEq)]
enum MapType {
    WorldMap,   // 大地图
    Town,       // 城镇
    Dungeon,    // 地牢
}

impl TileType {
    /// 将地砖类型转换为对应的UTF-8字符表示
    /// 使用标准 Roguelike 字符风格
    fn as_char(&self) -> &str {
        match self {
            TileType::Floor => ".",      // 地板用点表示
            TileType::Wall => "#",       // 墙壁用井号表示
            TileType::Door => "+",       // 门用加号表示
            TileType::Water => "~",      // 水域用波浪号表示
            TileType::Grass => "\"",     // 草地用双引号表示
            TileType::Mountain => "^",   // 山脉用尖号表示
            TileType::Forest => "♠",     // 森林用黑桃表示
            TileType::Town => "※",      // 城镇用米字表示
            TileType::Dungeon => "▼",    // 地牢用三角表示
        }
    }
    
    /// 判断该地砖类型是否可以行走
    /// 返回 true 表示玩家可以通过该地砖
    fn is_walkable(&self) -> bool {
        matches!(self, 
            TileType::Floor | 
            TileType::Door | 
            TileType::Grass | 
            TileType::Forest |
            TileType::Town |
            TileType::Dungeon
        )
    }
    
    /// 判断是否为可进入的地点（城镇或地牢）
    fn is_enterable(&self) -> bool {
        matches!(self, TileType::Town | TileType::Dungeon)
    }
}

/// 物品结构体
/// 表示游戏中可以拾取的物品
#[derive(Clone)]
struct Item {
    name: String,      // 物品名称
    char: &'static str, // 物品在地图上显示的字符
    item_type: ItemType, // 物品类型（武器、护甲、消耗品等）
}

/// 物品类型枚举
/// 定义不同种类的物品及其属性
#[derive(Clone)]
enum ItemType {
    Weapon { damage: i32 },      // 武器 - 附带伤害值
    Armor { defense: i32 },      // 护甲 - 附带防御值
    Consumable { heal: i32 },    // 消耗品 - 附带治疗值
    Quest,                        // 任务物品
}

/// NPC（非玩家角色）结构体
#[derive(Clone)]
struct NPC {
    name: String,           // NPC名称
    char: &'static str,     // NPC在地图上显示的字符
    x: i32,                 // NPC的X坐标
    y: i32,                 // NPC的Y坐标
    hp: i32,                // 当前生命值
    max_hp: i32,            // 最大生命值
    hostile: bool,          // 是否敌对（true为敌人，false为友好）
    dialogue: Vec<String>,  // 对话内容列表
}

/// 玩家结构体
struct Player {
    x: i32,                      // 玩家的X坐标
    y: i32,                      // 玩家的Y坐标
    hp: i32,                     // 当前生命值
    max_hp: i32,                 // 最大生命值
    inventory: Vec<Item>,        // 背包物品列表
    stats: PlayerStats,          // 玩家属性点
}

/// 玩家属性结构体
/// 模仿辐射系列的SPECIAL系统
struct PlayerStats {
    strength: i32,      // 力量 - 影响近战伤害和负重
    perception: i32,    // 感知 - 影响远程精准度
    endurance: i32,     // 耐力 - 影响生命值和抗性
    charisma: i32,      // 魅力 - 影响对话选项
    intelligence: i32,  // 智力 - 影响技能点
    agility: i32,       // 敏捷 - 影响行动点数
    luck: i32,          // 幸运 - 影响暴击率
}

/// 游戏地图结构体
#[derive(Clone)]
struct GameMap {
    width: i32,                          // 地图宽度
    height: i32,                         // 地图高度
    tiles: Vec<Vec<TileType>>,           // 二维地砖数组
    items: HashMap<(i32, i32), Item>,    // 物品位置映射表 (坐标 -> 物品)
    map_type: MapType,                   // 地图类型
    name: String,                        // 地图名称
}

impl GameMap {
    /// 创建世界大地图
    fn new_world_map() -> Self {
        let width = 80;
        let height = 40;
        let mut tiles = vec![vec![TileType::Grass; width as usize]; height as usize];
        
        // 添加山脉
        for y in 5..10 {
            for x in 20..30 {
                tiles[y][x] = TileType::Mountain;
            }
        }
        
        // 添加森林
        for y in 15..25 {
            for x in 10..20 {
                tiles[y][x] = TileType::Forest;
            }
        }
        
        // 添加水域
        for y in 30..35 {
            for x in 40..60 {
                tiles[y][x] = TileType::Water;
            }
        }
        
        // 放置城镇入口
        tiles[10][15] = TileType::Town;
        tiles[25][50] = TileType::Town;
        
        // 放置地牢入口
        tiles[8][40] = TileType::Dungeon;
        tiles[30][25] = TileType::Dungeon;
        
        GameMap {
            width,
            height,
            tiles,
            items: HashMap::new(),
            map_type: MapType::WorldMap,
            name: "Wasteland".to_string(),
        }
    }
    
    /// 创建城镇地图
    fn new_town_map(town_id: usize) -> Self {
        let width = 40;
        let height = 30;
        let mut tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        
        // 创建边界墙
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                }
            }
        }
        
        // 创建建筑物（房间）
        for x in 5..15 {
            for y in 5..12 {
                tiles[y][x] = TileType::Wall;
            }
        }
        tiles[8][10] = TileType::Door;  // 门
        
        for x in 20..30 {
            for y in 15..22 {
                tiles[y][x] = TileType::Wall;
            }
        }
        tiles[18][25] = TileType::Door;
        
        // 添加一些装饰性水域（井或喷泉）
        tiles[15][10] = TileType::Water;
        
        let mut items = HashMap::new();
        items.insert((10, 15), Item {
            name: "Town Supply".to_string(),
            char: "$",
            item_type: ItemType::Consumable { heal: 30 },
        });
        
        GameMap {
            width,
            height,
            tiles,
            items,
            map_type: MapType::Town,
            name: format!("Town #{}", town_id + 1),
        }
    }
    
    /// 创建地牢地图
    fn new_dungeon_map(dungeon_id: usize) -> Self {
        let width = 40;
        let height = 30;
        let mut tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        
        // 创建迷宫般的地牢布局
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                }
            }
        }
        
        // 添加内部墙壁创建走廊
        for x in 10..15 {
            tiles[5][x] = TileType::Wall;
        }
        tiles[5][12] = TileType::Door;
        
        for y in 10..20 {
            tiles[y][20] = TileType::Wall;
        }
        tiles[15][20] = TileType::Door;
        
        // 添加水域/岩浆
        for x in 25..30 {
            for y in 8..12 {
                tiles[y][x] = TileType::Water;
            }
        }
        
        let mut items = HashMap::new();
        items.insert((5, 5), Item {
            name: "Treasure Chest".to_string(),
            char: "☐",
            item_type: ItemType::Weapon { damage: 25 },
        });
        
        GameMap {
            width,
            height,
            tiles,
            items,
            map_type: MapType::Dungeon,
            name: format!("Dungeon #{}", dungeon_id + 1),
        }
    }
    
    /// 检查指定坐标是否可以行走
    /// 
    /// # 参数
    /// * `x` - X坐标
    /// * `y` - Y坐标
    /// 
    /// # 返回
    /// true 表示该位置可以行走，false 表示不可行走（墙壁、水域或超出边界）
    fn is_walkable(&self, x: i32, y: i32) -> bool {
        // 检查坐标是否在地图范围内
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        // 检查该位置的地砖类型是否可通行
        self.tiles[y as usize][x as usize].is_walkable()
    }
}

/// 游戏状态枚举
/// 定义游戏当前处于哪种模式
enum GameState {
    Playing,           // 正常游玩状态（移动、探索）
    Inventory,         // 背包界面
    Dialogue(usize),   // 对话状态（参数是NPC的索引）
    Combat(usize),     // 战斗状态（参数是敌人NPC的索引）
}

/// 地图位置记录
/// 用于在不同地图间切换时保存玩家位置
#[derive(Clone)]
struct MapLocation {
    map_type: MapType,   // 地图类型
    map_id: usize,       // 地图ID（用于区分不同的城镇/地牢）
    x: i32,              // 进入时的X坐标
    y: i32,              // 进入时的Y坐标
}

/// 游戏主结构体
/// 包含所有游戏数据和状态
struct Game {
    player: Player,              // 玩家数据
    current_map: GameMap,        // 当前地图
    world_map: GameMap,          // 世界大地图（缓存）
    town_maps: Vec<GameMap>,     // 城镇地图列表
    dungeon_maps: Vec<GameMap>,  // 地牢地图列表
    npcs: Vec<NPC>,              // 当前地图的NPC列表
    state: GameState,            // 当前游戏状态
    messages: Vec<String>,       // 消息日志（最多显示5条）
    camera_x: i32,               // 摄像机X坐标（用于地图滚动）
    camera_y: i32,               // 摄像机Y坐标（用于地图滚动）
    previous_location: Option<MapLocation>,  // 进入小地图前的位置
}

impl Game {
    /// 创建新游戏实例
    /// 初始化玩家、地图、NPC等所有游戏元素
    fn new() -> Self {
        // 创建玩家角色，初始位置在世界地图 (40, 20)
        let player = Player {
            x: 40,
            y: 20,
            hp: 100,
            max_hp: 100,
            inventory: vec![],  // 初始背包为空
            stats: PlayerStats {
                // 初始属性点均为5
                strength: 5,
                perception: 5,
                endurance: 5,
                charisma: 5,
                intelligence: 5,
                agility: 5,
                luck: 5,
            },
        };
        
        // 创建世界大地图
        let world_map = GameMap::new_world_map();
        
        // 预生成城镇地图
        let town_maps = vec![
            GameMap::new_town_map(0),
            GameMap::new_town_map(1),
        ];
        
        // 预生成地牢地图
        let dungeon_maps = vec![
            GameMap::new_dungeon_map(0),
            GameMap::new_dungeon_map(1),
        ];
        
        // 当前地图初始为世界地图
        let current_map = world_map.clone();
        
        // 创建NPC列表（世界地图上的NPC）
        let npcs = vec![
            NPC {
                name: "Traveling Merchant".to_string(),
                char: "♥",
                x: 35,
                y: 20,
                hp: 50,
                max_hp: 50,
                hostile: false,
                dialogue: vec![
                    "Hello, traveler!".to_string(),
                    "I sell goods across the wasteland.".to_string(),
                    "Safe travels!".to_string(),
                ],
            },
        ];
        
        Game {
            player,
            current_map,
            world_map,
            town_maps,
            dungeon_maps,
            npcs,
            state: GameState::Playing,
            messages: vec!["Welcome to the Wasteland! Press SPACE to enter towns/dungeons, ESC to return.".to_string()],
            camera_x: 0,
            camera_y: 0,
            previous_location: None,
        }
    }
    
    /// 添加消息到消息日志
    /// 如果消息超过5条，自动删除最旧的消息
    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        if self.messages.len() > 5 {
            self.messages.remove(0);  // 删除第一条（最旧的）消息
        }
    }
    
    /// 移动玩家
    /// 
    /// # 参数
    /// * `dx` - X轴移动增量（-1左移，1右移）
    /// * `dy` - Y轴移动增量（-1上移，1下移）
    fn move_player(&mut self, dx: i32, dy: i32) {
        let new_x = self.player.x + dx;
        let new_y = self.player.y + dy;
        
        // 检查目标位置是否有NPC
        if let Some(npc_idx) = self.npcs.iter().position(|n| n.x == new_x && n.y == new_y) {
            // 根据NPC的敌对性决定触发战斗还是对话
            if self.npcs[npc_idx].hostile {
                self.state = GameState::Combat(npc_idx);
                self.add_message(format!("Combat with {}!", self.npcs[npc_idx].name));
            } else {
                self.state = GameState::Dialogue(npc_idx);
            }
            return;  // 不移动玩家位置
        }
        
        // 检查地图碰撞（墙壁、水域等）
        if self.current_map.is_walkable(new_x, new_y) {
            // 更新玩家位置
            self.player.x = new_x;
            self.player.y = new_y;
            
            // 检查是否有物品可以拾取
            if let Some(item) = self.current_map.items.remove(&(new_x, new_y)) {
                self.add_message(format!("Picked up {}", item.name));
                self.player.inventory.push(item);  // 将物品添加到背包
            }
        }
    }
    
    /// 尝试进入城镇或地牢
    fn try_enter_location(&mut self) {
        let x = self.player.x;
        let y = self.player.y;
        
        // 只能在世界地图上进入城镇/地牢
        if self.current_map.map_type != MapType::WorldMap {
            return;
        }
        
        let tile = self.current_map.tiles[y as usize][x as usize];
        if !tile.is_enterable() {
            return;
        }
        
        // 保存当前位置
        self.previous_location = Some(MapLocation {
            map_type: MapType::WorldMap,
            map_id: 0,
            x,
            y,
        });
        
        // 根据地砖类型进入不同的地图
        match tile {
            TileType::Town => {
                // 根据位置确定进入哪个城镇
                let town_id = if (x, y) == (15, 10) { 0 } else { 1 };
                self.current_map = self.town_maps[town_id].clone();
                self.player.x = 20;
                self.player.y = 15;
                self.load_town_npcs(town_id);
                self.add_message(format!("Entered {}", self.current_map.name));
            }
            TileType::Dungeon => {
                // 根据位置确定进入哪个地牢
                let dungeon_id = if (x, y) == (40, 8) { 0 } else { 1 };
                self.current_map = self.dungeon_maps[dungeon_id].clone();
                self.player.x = 5;
                self.player.y = 5;
                self.load_dungeon_npcs(dungeon_id);
                self.add_message(format!("Entered {}", self.current_map.name));
            }
            _ => {}
        }
    }
    
    /// 返回世界地图
    fn return_to_world_map(&mut self) {
        if self.current_map.map_type == MapType::WorldMap {
            return;  // 已经在世界地图上
        }
        
        if let Some(prev_loc) = &self.previous_location {
            self.current_map = self.world_map.clone();
            self.player.x = prev_loc.x;
            self.player.y = prev_loc.y;
            self.previous_location = None;
            
            // 加载世界地图NPC
            self.load_world_npcs();
            self.add_message("Returned to world map".to_string());
        }
    }
    
    /// 加载世界地图NPC
    fn load_world_npcs(&mut self) {
        self.npcs = vec![
            NPC {
                name: "Traveling Merchant".to_string(),
                char: "♥",
                x: 35,
                y: 20,
                hp: 50,
                max_hp: 50,
                hostile: false,
                dialogue: vec![
                    "Hello, traveler!".to_string(),
                    "I sell goods across the wasteland.".to_string(),
                    "Safe travels!".to_string(),
                ],
            },
        ];
    }
    
    /// 加载城镇NPC
    fn load_town_npcs(&mut self, _town_id: usize) {
        self.npcs = vec![
            NPC {
                name: "Townfolk".to_string(),
                char: "☺",
                x: 15,
                y: 15,
                hp: 50,
                max_hp: 50,
                hostile: false,
                dialogue: vec![
                    "Welcome to our town!".to_string(),
                    "It's safe here.".to_string(),
                ],
            },
            NPC {
                name: "Blacksmith".to_string(),
                char: "♦",
                x: 10,
                y: 8,
                hp: 80,
                max_hp: 80,
                hostile: false,
                dialogue: vec![
                    "Need repairs?".to_string(),
                    "My craftsmanship is top notch!".to_string(),
                ],
            },
        ];
    }
    
    /// 加载地牢NPC（敌人）
    fn load_dungeon_npcs(&mut self, _dungeon_id: usize) {
        self.npcs = vec![
            NPC {
                name: "Dungeon Guard".to_string(),
                char: "G",
                x: 10,
                y: 10,
                hp: 80,
                max_hp: 80,
                hostile: true,
                dialogue: vec!["Intruders must die!".to_string()],
            },
            NPC {
                name: "Mutant Beast".to_string(),
                char: "M",
                x: 25,
                y: 15,
                hp: 100,
                max_hp: 100,
                hostile: true,
                dialogue: vec!["Hssssss...".to_string()],
            },
        ];
    }
    
    /// 更新摄像机位置，使其跟随玩家
    /// 摄像机会保持玩家在屏幕中央附近
    fn update_camera(&mut self) {
        // 将摄像机中心对准玩家位置
        // 偏移量根据视野大小调整（20格宽，10格高）
        self.camera_x = self.player.x - 20;
        self.camera_y = self.player.y - 10;
    }
}

// ========== 渲染系统 ==========

/// 绘制游戏主界面（地图、物品、NPC、玩家）
fn draw_game(game: &Game, font: &Font) {
    let tile_size = 20.0;   // 每个格子的像素大小
    let start_x = 20.0;     // 地图绘制的起始X坐标
    let start_y = 40.0;     // 地图绘制的起始Y坐标
    
    // 绘制地图的所有地砖
    for y in 0..game.current_map.height {
        for x in 0..game.current_map.width {
            // 计算地砖在屏幕上的位置（考虑摄像机偏移）
            let screen_x = start_x + (x - game.camera_x) as f32 * tile_size;
            let screen_y = start_y + (y - game.camera_y) as f32 * tile_size;
            
            // 如果地砖不在屏幕可见区域内，跳过绘制
            if screen_x < 0.0 || screen_y < 0.0 || screen_x > screen_width() || screen_y > screen_height() {
                continue;
            }
            
            // 获取地砖类型并设置对应颜色
            let tile = game.current_map.tiles[y as usize][x as usize];
            let color = match tile {
                TileType::Floor => DARKGRAY,     // 地板：深灰色
                TileType::Wall => GRAY,          // 墙壁：灰色
                TileType::Door => BROWN,         // 门：棕色
                TileType::Water => BLUE,         // 水域：蓝色
                TileType::Grass => DARKGREEN,    // 草地：深绿色
                TileType::Mountain => LIGHTGRAY, // 山脉：浅灰色
                TileType::Forest => GREEN,       // 森林：绿色
                TileType::Town => ORANGE,        // 城镇：橙色
                TileType::Dungeon => DARKPURPLE, // 地牢：深紫色
            };
            
            // 绘制地砖矩形背景
            draw_rectangle(screen_x, screen_y, tile_size, tile_size, color);
            
            // 绘制地砖的ASCII字符
            draw_text_ex(
                tile.as_char(),
                screen_x + 5.0,
                screen_y + 15.0,
                TextParams {
                    font: Some(font),
                    font_size: 20,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }
    }
    
    // 绘制地图上的物品
    for ((x, y), item) in &game.current_map.items {
        // 计算物品在屏幕上的位置
        let screen_x = start_x + (*x - game.camera_x) as f32 * tile_size;
        let screen_y = start_y + (*y - game.camera_y) as f32 * tile_size;
        
        // 用黄色绘制物品字符
        draw_text_ex(
            item.char,
            screen_x + 5.0,
            screen_y + 15.0,
            TextParams {
                font: Some(font),
                font_size: 20,
                color: YELLOW,
                ..Default::default()
            },
        );
    }
    
    // 绘制所有NPC
    for npc in &game.npcs {
        // 计算NPC在屏幕上的位置
        let screen_x = start_x + (npc.x - game.camera_x) as f32 * tile_size;
        let screen_y = start_y + (npc.y - game.camera_y) as f32 * tile_size;
        
        // 根据敌对性设置颜色：红色为敌人，绿色为友好
        let color = if npc.hostile { RED } else { GREEN };
        
        // 绘制NPC字符
        draw_text_ex(
            npc.char,
            screen_x + 5.0,
            screen_y + 15.0,
            TextParams {
                font: Some(font),
                font_size: 20,
                color,
                ..Default::default()
            },
        );
    }
    
    // 绘制玩家角色（用 @ 符号表示）
    let player_screen_x = start_x + (game.player.x - game.camera_x) as f32 * tile_size;
    let player_screen_y = start_y + (game.player.y - game.camera_y) as f32 * tile_size;
    draw_text_ex(
        "@",
        player_screen_x + 5.0,
        player_screen_y + 15.0,
        TextParams {
            font: Some(font),
            font_size: 20,
            color: SKYBLUE,
            ..Default::default()
        },
    );
}

/// 绘制用户界面（状态栏、消息日志、控制提示）
fn draw_ui(game: &Game, font: &Font) {
    // === 绘制顶部状态栏 ===
    // 黑色背景
    draw_rectangle(0.0, 0.0, screen_width(), 30.0, BLACK);
    
    // 显示玩家状态信息和当前地图
    draw_text_ex(
        &format!("HP: {}/{} | Pos: ({},{}) | Items: {} | Map: {}", 
                 game.player.hp, game.player.max_hp,
                 game.player.x, game.player.y,
                 game.player.inventory.len(),
                 game.current_map.name),
        10.0, 20.0,
        TextParams {
            font: Some(font),
            font_size: 20,
            color: GREEN,
            ..Default::default()
        }
    );
    
    // === 绘制底部消息日志 ===
    let log_y = screen_height() - 120.0;
    // 半透明黑色背景
    draw_rectangle(0.0, log_y, screen_width(), 120.0, Color::new(0.0, 0.0, 0.0, 0.8));
    
    // 显示最近的5条消息
    for (i, msg) in game.messages.iter().enumerate() {
        draw_text_ex(
            msg, 
            10.0, 
            log_y + 20.0 + i as f32 * 20.0,
            TextParams {
                font: Some(font),
                font_size: 18,
                color: LIGHTGRAY,
                ..Default::default()
            }
        );
    }
    
    // === 绘制控制提示 ===
    let controls = if game.current_map.map_type == MapType::WorldMap {
        "WASD/Arrow: Move | Space: Enter Town/Dungeon | I: Inventory"
    } else {
        "WASD/Arrow: Move | ESC: Return to World | I: Inventory"
    };
    draw_text_ex(
        controls, 
        10.0, 
        screen_height() - 10.0,
        TextParams {
            font: Some(font),
            font_size: 16,
            color: DARKGRAY,
            ..Default::default()
        }
    );
}

/// 绘制背包界面
fn draw_inventory(game: &Game, font: &Font) {
    // 计算面板居中位置
    let panel_w = 400.0;
    let panel_h = 300.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    
    // 绘制面板背景和边框
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
    
    // 绘制标题
    draw_text_ex("INVENTORY", panel_x + 10.0, panel_y + 30.0, TextParams {
        font: Some(font),
        font_size: 24,
        color: YELLOW,
        ..Default::default()
    });
    
    // 显示背包内容
    if game.player.inventory.is_empty() {
        draw_text_ex("Empty", panel_x + 10.0, panel_y + 60.0, TextParams {
            font: Some(font),
            font_size: 20,
            color: GRAY,
            ..Default::default()
        });
    } else {
        // 列出所有物品
        for (i, item) in game.player.inventory.iter().enumerate() {
            draw_text_ex(
                &format!("{} - {}", item.char, item.name),
                panel_x + 10.0,
                panel_y + 60.0 + i as f32 * 25.0,
                TextParams {
                    font: Some(font),
                    font_size: 20,
                    color: WHITE,
                    ..Default::default()
                }
            );
        }
    }
    
    // 绘制关闭提示
    draw_text_ex("Press I to close", panel_x + 10.0, panel_y + panel_h - 20.0, TextParams {
        font: Some(font),
        font_size: 16,
        color: DARKGRAY,
        ..Default::default()
    });
}

/// 绘制对话界面
fn draw_dialogue(game: &Game, npc_idx: usize, font: &Font) {
    // 计算对话框位置（屏幕底部）
    let panel_w = 500.0;
    let panel_h = 200.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = screen_height() - panel_h - 50.0;
    
    // 绘制对话框背景和边框（绿色边框表示友好）
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, GREEN);
    
    // 获取NPC数据
    let npc = &game.npcs[npc_idx];
    
    // 显示NPC名字
    draw_text_ex(&npc.name, panel_x + 10.0, panel_y + 30.0, TextParams {
        font: Some(font),
        font_size: 22,
        color: GREEN,
        ..Default::default()
    });
    
    // 显示对话内容
    for (i, line) in npc.dialogue.iter().enumerate() {
        draw_text_ex(line, panel_x + 10.0, panel_y + 60.0 + i as f32 * 25.0, TextParams {
            font: Some(font),
            font_size: 18,
            color: WHITE,
            ..Default::default()
        });
    }
    
    // 绘制继续提示
    draw_text_ex("Press SPACE to continue", panel_x + 10.0, panel_y + panel_h - 20.0, TextParams {
        font: Some(font),
        font_size: 16,
        color: DARKGRAY,
        ..Default::default()
    });
}

/// 绘制战斗界面
fn draw_combat(game: &Game, npc_idx: usize, font: &Font) {
    // 计算战斗面板居中位置
    let panel_w = 500.0;
    let panel_h = 250.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    
    // 绘制战斗面板背景和边框（红色边框表示战斗）
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, RED);
    
    // 获取敌人数据
    let npc = &game.npcs[npc_idx];
    
    // 显示战斗标题
    draw_text_ex("COMBAT", panel_x + 10.0, panel_y + 30.0, TextParams {
        font: Some(font),
        font_size: 24,
        color: RED,
        ..Default::default()
    });
    
    // 显示敌人信息
    draw_text_ex(&format!("Enemy: {}", npc.name), panel_x + 10.0, panel_y + 60.0, TextParams {
        font: Some(font),
        font_size: 20,
        color: ORANGE,
        ..Default::default()
    });
    draw_text_ex(&format!("Enemy HP: {}/{}", npc.hp, npc.max_hp), 
              panel_x + 10.0, panel_y + 85.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: WHITE,
        ..Default::default()
    });
    
    // 显示玩家信息
    draw_text_ex(&format!("Your HP: {}/{}", game.player.hp, game.player.max_hp), 
              panel_x + 10.0, panel_y + 110.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: WHITE,
        ..Default::default()
    });
    
    // 显示战斗选项
    draw_text_ex("1: Attack", panel_x + 10.0, panel_y + 150.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: YELLOW,
        ..Default::default()
    });
    draw_text_ex("2: Use Item", panel_x + 10.0, panel_y + 175.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: YELLOW,
        ..Default::default()
    });
    draw_text_ex("3: Run", panel_x + 10.0, panel_y + 200.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: YELLOW,
        ..Default::default()
    });
}

// ========== 主循环 ==========

/// 游戏主循环
/// macroquad::main 宏处理窗口创建和事件循环
#[macroquad::main("Fallout-style RPG")]
async fn main() {
    // 加载自定义字体
    let font = load_ttf_font("assets/fonts/JetBrainsMonoNL-Regular.ttf")
        .await
        .expect("Failed to load font");
    
    // 创建游戏实例
    let mut game = Game::new();

    // 游戏主循环 - 每帧执行一次
    loop {
        // 清空屏幕为黑色
        clear_background(BLACK);

        // ========== 输入处理 ==========
        // 根据当前游戏状态处理不同的输入
        match game.state {
            // 游玩状态：处理移动和打开背包
            GameState::Playing => {
                // 上移：W键或方向键上
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    game.move_player(0, -1);
                }
                // 下移：S键或方向键下
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    game.move_player(0, 1);
                }
                // 左移：A键或方向键左
                if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                    game.move_player(-1, 0);
                }
                // 右移：D键或方向键右
                if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                    game.move_player(1, 0);
                }
                // 打开背包：I键
                if is_key_pressed(KeyCode::I) {
                    game.state = GameState::Inventory;
                }
                // 进入城镇/地牢：空格键
                if is_key_pressed(KeyCode::Space) {
                    game.try_enter_location();
                }
                // 返回世界地图：ESC键
                if is_key_pressed(KeyCode::Escape) {
                    game.return_to_world_map();
                }
            }
            
            // 背包状态：处理关闭背包
            GameState::Inventory => {
                // I键或ESC键关闭背包
                if is_key_pressed(KeyCode::I) || is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                }
            }
            
            // 对话状态：处理继续/退出对话
            GameState::Dialogue(_) => {
                // 空格键或ESC键结束对话
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                }
            }
            
            // 战斗状态：处理战斗选项
            GameState::Combat(npc_idx) => {
                // 选项1：攻击
                if is_key_pressed(KeyCode::Key1) {
                    // 计算伤害
                    let damage = 15;
                    game.npcs[npc_idx].hp -= damage;
                    game.add_message(format!("You dealt {} damage!", damage));
                    
                    // 检查敌人是否被击败
                    if game.npcs[npc_idx].hp <= 0 {
                        game.add_message(format!("{} defeated!", game.npcs[npc_idx].name));
                        game.npcs.remove(npc_idx);  // 从游戏中移除敌人
                        game.state = GameState::Playing;
                    } else {
                        // 敌人反击
                        let enemy_damage = 10;
                        game.player.hp -= enemy_damage;
                        game.add_message(format!("Enemy dealt {} damage!", enemy_damage));
                    }
                }
                
                // 选项3：逃跑
                if is_key_pressed(KeyCode::Key3) {
                    game.add_message("You ran away!".to_string());
                    game.state = GameState::Playing;
                }
            }
        }
        
        // ========== 更新游戏状态 ==========
        // 更新摄像机位置，跟随玩家
        game.update_camera();
        
        // ========== 渲染 ==========
        // 绘制游戏主界面（地图、NPC、玩家）
        draw_game(&game, &font);
        
        // 绘制UI元素（状态栏、消息日志）
        draw_ui(&game, &font);
        
        // 根据当前状态绘制额外的界面
        match game.state {
            GameState::Inventory => draw_inventory(&game, &font),         // 背包界面
            GameState::Dialogue(idx) => draw_dialogue(&game, idx, &font), // 对话界面
            GameState::Combat(idx) => draw_combat(&game, idx, &font),     // 战斗界面
            _ => {}  // Playing 状态不需要额外界面
        }
        
        // 等待下一帧（控制帧率，处理系统事件）
        next_frame().await;
    }
}