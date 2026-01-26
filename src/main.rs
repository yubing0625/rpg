
// Import macroquad game engine core functionality
// Includes graphics rendering, input handling, color definitions, etc.
use macroquad::prelude::*;

// Import HashMap for storing item positions on the map
use std::collections::HashMap;

// ========== Core Data Structures ==========

/// Tile type enumeration
/// Defines all possible terrain types in the game world
#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Floor,     // Floor - walkable
    Wall,      // Wall - not walkable
    Door,      // Door - walkable
    Water,     // Water - not walkable
    Grass,     // Grass - walkable (world map)
    Mountain,  // Mountain - not walkable (world map)
    Forest,    // Forest - walkable (world map)
    Town,      // Town entrance - enterable
    Dungeon,   // Dungeon entrance - enterable
}

/// Map type enumeration
/// Distinguishes between world map and small maps (towns/dungeons)
#[derive(Clone, Copy, PartialEq)]
enum MapType {
    WorldMap,   // World map
    Town,       // Town
    Dungeon,    // Dungeon
}

impl TileType {
    /// Convert tile type to corresponding UTF-8 character representation
    /// Uses standard Roguelike character style
    fn as_char(&self) -> &str {
        match self {
            TileType::Floor => ".",      // Floor represented by dot
            TileType::Wall => "#",       // Wall represented by hash
            TileType::Door => "+",       // Door represented by plus
            TileType::Water => "~",      // Water represented by tilde
            TileType::Grass => "\"",     // Grass represented by quote
            TileType::Mountain => "^",   // Mountain represented by caret
            TileType::Forest => "&",     // Forest represented by ampersand
            TileType::Town => "※",      // Town represented by asterisk
            TileType::Dungeon => "▼",    // Dungeon represented by triangle
        }
    }
    
    /// Check if this tile type is walkable
    /// Returns true if player can pass through this tile
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
    
    /// Check if this is an enterable location (town or dungeon)
    fn is_enterable(&self) -> bool {
        matches!(self, TileType::Town | TileType::Dungeon)
    }
}

/// Item structure
/// Represents items that can be picked up in the game
#[derive(Clone)]
struct Item {
    name: String,      // Item name
    char: &'static str, // Character displayed on map
    item_type: ItemType, // Item type (weapon, armor, consumable, etc.)
}

/// Item type enumeration
/// Defines different kinds of items and their attributes
#[derive(Clone)]
enum ItemType {
    Weapon { damage: i32 },      // Weapon - with damage value
    Armor { defense: i32 },      // Armor - with defense value
    Consumable { heal: i32 },    // Consumable - with heal value
    Quest,                        // Quest item
}

/// Dialogue option structure
#[derive(Clone)]
struct DialogueOption {
    text: String,           // Option text
    next_node: Option<usize>, // Next node to jump to (None means end dialogue)
}

/// Dialogue node structure
#[derive(Clone)]
struct DialogueNode {
    text: String,                  // Current node's dialogue text
    options: Vec<DialogueOption>,  // Available options
}

/// NPC (Non-Player Character) structure
#[derive(Clone)]
struct NPC {
    name: String,           // NPC name
    char: &'static str,     // Character displayed on map
    x: i32,                 // NPC X coordinate
    y: i32,                 // NPC Y coordinate
    hp: i32,                // Current health
    max_hp: i32,            // Maximum health
    hostile: bool,          // Whether hostile (true = enemy, false = friendly)
    dialogue: Vec<DialogueNode>,  // Branching dialogue tree
}

/// Player structure
struct Player {
    x: i32,                      // Player X coordinate
    y: i32,                      // Player Y coordinate
    hp: i32,                     // Current health
    max_hp: i32,                 // Maximum health
    inventory: Vec<Item>,        // Inventory item list
    stats: PlayerStats,          // Player attributes
}

/// Player stats structure
/// Mimics Fallout series SPECIAL system
struct PlayerStats {
    strength: i32,      // Strength - affects melee damage and carry weight
    perception: i32,    // Perception - affects ranged accuracy
    endurance: i32,     // Endurance - affects health and resistance
    charisma: i32,      // Charisma - affects dialogue options
    intelligence: i32,  // Intelligence - affects skill points
    agility: i32,       // Agility - affects action points
    luck: i32,          // Luck - affects critical hit rate
}

/// Game map structure
#[derive(Clone)]
struct GameMap {
    width: i32,                          // Map width
    height: i32,                         // Map height
    tiles: Vec<Vec<TileType>>,           // 2D tile array
    items: HashMap<(i32, i32), Item>,    // Item position mapping (coordinates -> item)
    map_type: MapType,                   // Map type
    name: String,                        // Map name
}

impl GameMap {
    /// Create world map
    fn new_world_map() -> Self {
        let width = 80;
        let height = 40;
        let mut tiles = vec![vec![TileType::Grass; width as usize]; height as usize];
        
        // Add mountains
        for y in 5..10 {
            for x in 20..30 {
                tiles[y][x] = TileType::Mountain;
            }
        }
        
        // Add forests
        for y in 15..25 {
            for x in 10..20 {
                tiles[y][x] = TileType::Forest;
            }
        }
        
        // Add water
        for y in 30..35 {
            for x in 40..60 {
                tiles[y][x] = TileType::Water;
            }
        }
        
        // Place town entrances
        tiles[10][15] = TileType::Town;
        tiles[25][50] = TileType::Town;
        
        // Place dungeon entrances
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
    
    /// Create town map
    fn new_town_map(town_id: usize) -> Self {
        let width = 40;
        let height = 30;
        let mut tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        
        // Create boundary walls
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                }
            }
        }
        
        // Create buildings (rooms)
        for x in 5..15 {
            for y in 5..12 {
                tiles[y][x] = TileType::Wall;
            }
        }
        tiles[8][10] = TileType::Door;  // Door
        
        for x in 20..30 {
            for y in 15..22 {
                tiles[y][x] = TileType::Wall;
            }
        }
        tiles[18][25] = TileType::Door;
        
        // Add decorative water (well or fountain)
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
    
    /// Create dungeon map
    fn new_dungeon_map(dungeon_id: usize) -> Self {
        let width = 40;
        let height = 30;
        let mut tiles = vec![vec![TileType::Floor; width as usize]; height as usize];
        
        // Create maze-like dungeon layout
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                }
            }
        }
        
        // Add interior walls to create corridors
        for x in 10..15 {
            tiles[5][x] = TileType::Wall;
        }
        tiles[5][12] = TileType::Door;
        
        for y in 10..20 {
            tiles[y][20] = TileType::Wall;
        }
        tiles[15][20] = TileType::Door;
        
        // Add water/lava
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
    
    /// Check if the specified coordinates are walkable
    /// 
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// 
    /// # Returns
    /// true if the position is walkable, false if not (wall, water, or out of bounds)
    fn is_walkable(&self, x: i32, y: i32) -> bool {
        // Check if coordinates are within map bounds
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        // Check if the tile type at this position is passable
        self.tiles[y as usize][x as usize].is_walkable()
    }
}

/// Game state enumeration
/// Defines which mode the game is currently in
enum GameState {
    Playing,           // Normal gameplay state (movement, exploration)
    Inventory,         // Inventory interface
    Dialogue(usize, usize, usize),   // Dialogue state (NPC index, current node index, selected option index)
    Combat(usize),     // Combat state (enemy NPC index)
}

/// Map location record
/// Used to save player position when switching between maps
#[derive(Clone)]
struct MapLocation {
    map_type: MapType,   // Map type
    map_id: usize,       // Map ID (to distinguish different towns/dungeons)
    x: i32,              // X coordinate when entering
    y: i32,              // Y coordinate when entering
}

/// Main game structure
/// Contains all game data and state
struct Game {
    player: Player,              // Player data
    current_map: GameMap,        // Current map
    world_map: GameMap,          // World map (cached)
    town_maps: Vec<GameMap>,     // Town map list
    dungeon_maps: Vec<GameMap>,  // Dungeon map list
    npcs: Vec<NPC>,              // NPC list for current map
    state: GameState,            // Current game state
    messages: Vec<String>,       // Message log (max 5 messages)
    camera_x: i32,               // Camera X coordinate (for map scrolling)
    camera_y: i32,               // Camera Y coordinate (for map scrolling)
    previous_location: Option<MapLocation>,  // Position before entering small map
}

impl Game {
    /// Create new game instance
    /// Initialize player, maps, NPCs and all game elements
    fn new() -> Self {
        // Create player character, initial position at world map (40, 20)
        let player = Player {
            x: 40,
            y: 20,
            hp: 100,
            max_hp: 100,
            inventory: vec![],  // Initial inventory is empty
            stats: PlayerStats {
                // Initial stat points all set to 5
                strength: 5,
                perception: 5,
                endurance: 5,
                charisma: 5,
                intelligence: 5,
                agility: 5,
                luck: 5,
            },
        };
        
        // Create world map
        let world_map = GameMap::new_world_map();
        
        // Pre-generate town maps
        let town_maps = vec![
            GameMap::new_town_map(0),
            GameMap::new_town_map(1),
        ];
        
        // Pre-generate dungeon maps
        let dungeon_maps = vec![
            GameMap::new_dungeon_map(0),
            GameMap::new_dungeon_map(1),
        ];
        
        // Current map initially is world map
        let current_map = world_map.clone();
        
        // Create NPC list (NPCs on world map)
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
                    DialogueNode {
                        text: "Howdy, stranger! What brings you to these parts?".to_string(),
                        options: vec![
                            DialogueOption { text: "I'm here for adventure!".to_string(), next_node: Some(1) },
                            DialogueOption { text: "Just passing by.".to_string(), next_node: Some(2) },
                            DialogueOption { text: "None of your business.".to_string(), next_node: None },
                        ],
                    },
                    DialogueNode {
                        text: "Adventure, eh? Well, watch out for demonic cows!".to_string(),
                        options: vec![
                            DialogueOption { text: "Thanks for the tip!".to_string(), next_node: None },
                        ],
                    },
                    DialogueNode {
                        text: "Safe travels, partner!".to_string(),
                        options: vec![
                            DialogueOption { text: "See ya!".to_string(), next_node: None },
                        ],
                    },
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
    
    /// Add message to message log
    /// Automatically removes oldest message if exceeds 5 messages
    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        if self.messages.len() > 5 {
            self.messages.remove(0);  // Remove first (oldest) message
        }
    }
    
    /// Move player
    /// 
    /// # Arguments
    /// * `dx` - X axis movement delta (-1 left, 1 right)
    /// * `dy` - Y axis movement delta (-1 up, 1 down)
    fn move_player(&mut self, dx: i32, dy: i32) {
        let new_x = self.player.x + dx;
        let new_y = self.player.y + dy;
        
        // Check if there's an NPC at target position
        if let Some(npc_idx) = self.npcs.iter().position(|n| n.x == new_x && n.y == new_y) {
            // Trigger combat or dialogue based on NPC hostility
            if self.npcs[npc_idx].hostile {
                self.state = GameState::Combat(npc_idx);
                self.add_message(format!("Combat with {}!", self.npcs[npc_idx].name));
            } else {
                self.state = GameState::Dialogue(npc_idx, 0, 0); // Start from node 0, option 0 selected
            }
            return;  // Don't move player position
        }
        
        // Check map collision (walls, water, etc.)
        if self.current_map.is_walkable(new_x, new_y) {
            // Update player position
            self.player.x = new_x;
            self.player.y = new_y;
            
            // Check if there's an item to pick up
            if let Some(item) = self.current_map.items.remove(&(new_x, new_y)) {
                self.add_message(format!("Picked up {}", item.name));
                self.player.inventory.push(item);  // Add item to inventory
            }
        }
    }
    
    /// Try to enter town or dungeon
    fn try_enter_location(&mut self) {
        let x = self.player.x;
        let y = self.player.y;
        
        // Can only enter towns/dungeons from world map
        if self.current_map.map_type != MapType::WorldMap {
            return;
        }
        
        let tile = self.current_map.tiles[y as usize][x as usize];
        if !tile.is_enterable() {
            return;
        }
        
        // Save current position
        self.previous_location = Some(MapLocation {
            map_type: MapType::WorldMap,
            map_id: 0,
            x,
            y,
        });
        
        // Enter different maps based on tile type
        match tile {
            TileType::Town => {
                // Determine which town to enter based on position
                let town_id = if (x, y) == (15, 10) { 0 } else { 1 };
                self.current_map = self.town_maps[town_id].clone();
                self.player.x = 20;
                self.player.y = 15;
                self.load_town_npcs(town_id);
                self.add_message(format!("Entered {}", self.current_map.name));
            }
            TileType::Dungeon => {
                // Determine which dungeon to enter based on position
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
    
    /// Return to world map
    fn return_to_world_map(&mut self) {
        if self.current_map.map_type == MapType::WorldMap {
            return;  // Already on world map
        }
        
        if let Some(prev_loc) = &self.previous_location {
            self.current_map = self.world_map.clone();
            self.player.x = prev_loc.x;
            self.player.y = prev_loc.y;
            self.previous_location = None;
            
            // Load world map NPCs
            self.load_world_npcs();
            self.add_message("Returned to world map".to_string());
        }
    }
    
    /// Load world map NPCs
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
                    DialogueNode {
                        text: "Howdy, stranger! What brings you to these parts?".to_string(),
                        options: vec![
                            DialogueOption { text: "I'm here for adventure!".to_string(), next_node: Some(1) },
                            DialogueOption { text: "Just passing by.".to_string(), next_node: Some(2) },
                            DialogueOption { text: "None of your business.".to_string(), next_node: None },
                        ],
                    },
                    DialogueNode {
                        text: "Adventure, eh? Well, watch out for demonic cows!".to_string(),
                        options: vec![
                            DialogueOption { text: "Thanks for the tip!".to_string(), next_node: None },
                        ],
                    },
                    DialogueNode {
                        text: "Safe travels, partner!".to_string(),
                        options: vec![
                            DialogueOption { text: "See ya!".to_string(), next_node: None },
                        ],
                    },
                ],
            },
        ];
    }
    
    /// Load town NPCs
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
                    DialogueNode {
                        text: "Welcome to our town! Are you lost or just weird?".to_string(),
                        options: vec![
                            DialogueOption { text: "A bit of both, honestly.".to_string(), next_node: Some(1) },
                            DialogueOption { text: "I'm looking for work.".to_string(), next_node: Some(2) },
                        ],
                    },
                    DialogueNode {
                        text: "That's the spirit! You'll fit right in.".to_string(),
                        options: vec![
                            DialogueOption { text: "Thanks?".to_string(), next_node: None },
                        ],
                    },
                    DialogueNode {
                        text: "Try the saloon. Or the cemetery. Both are lively.".to_string(),
                        options: vec![
                            DialogueOption { text: "I'll check them out.".to_string(), next_node: None },
                        ],
                    },
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
                    DialogueNode {
                        text: "Need repairs? Or just here to chat?".to_string(),
                        options: vec![
                            DialogueOption { text: "My gear's busted.".to_string(), next_node: Some(1) },
                            DialogueOption { text: "Just lonely.".to_string(), next_node: Some(2) },
                        ],
                    },
                    DialogueNode {
                        text: "That'll be 50 meat. Up front.".to_string(),
                        options: vec![
                            DialogueOption { text: "Here you go.".to_string(), next_node: None },
                        ],
                    },
                    DialogueNode {
                        text: "Me too, friend. Me too.".to_string(),
                        options: vec![
                            DialogueOption { text: "...".to_string(), next_node: None },
                        ],
                    },
                ],
            },
        ];
    }
    
    /// Load dungeon NPCs (enemies)
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
                dialogue: vec![
                    DialogueNode {
                        text: "Intruders must die!".to_string(),
                        options: vec![
                            DialogueOption { text: "Fight!".to_string(), next_node: None },
                        ],
                    },
                ],
            },
            NPC {
                name: "Mutant Beast".to_string(),
                char: "M",
                x: 25,
                y: 15,
                hp: 100,
                max_hp: 100,
                hostile: true,
                dialogue: vec![
                    DialogueNode {
                        text: "Hssssss...".to_string(),
                        options: vec![
                            DialogueOption { text: "Back away slowly...".to_string(), next_node: None },
                        ],
                    },
                ],
            },
        ];
    }
    
    /// Update camera position to follow player
    /// Camera keeps player near center of screen
    fn update_camera(&mut self) {
        // Center camera on player position
        // Offset adjusted for viewport size (20 tiles wide, 10 tiles high)
        self.camera_x = self.player.x - 20;
        self.camera_y = self.player.y - 10;
    }
}

// ========== Rendering System ==========

/// Draw main game interface (map, items, NPCs, player)
fn draw_game(game: &Game, font: &Font) {
    let tile_size = 20.0;   // Pixel size of each tile
    let start_x = 20.0;     // Map drawing start X coordinate
    let start_y = 40.0;     // Map drawing start Y coordinate
    
    // Draw all map tiles
    for y in 0..game.current_map.height {
        for x in 0..game.current_map.width {
            // Calculate tile's screen position (accounting for camera offset)
            let screen_x = start_x + (x - game.camera_x) as f32 * tile_size;
            let screen_y = start_y + (y - game.camera_y) as f32 * tile_size;
            
            // Skip drawing if tile is outside visible screen area
            if screen_x < 0.0 || screen_y < 0.0 || screen_x > screen_width() || screen_y > screen_height() {
                continue;
            }
            
            // Get tile type and set corresponding color
            let tile = game.current_map.tiles[y as usize][x as usize];
            let color = match tile {
                TileType::Floor => DARKGRAY,     // Floor: dark gray
                TileType::Wall => GRAY,          // Wall: gray
                TileType::Door => BROWN,         // Door: brown
                TileType::Water => BLUE,         // Water: blue
                TileType::Grass => DARKGREEN,    // Grass: dark green
                TileType::Mountain => LIGHTGRAY, // Mountain: light gray
                TileType::Forest => GREEN,       // Forest: green
                TileType::Town => ORANGE,        // Town: orange
                TileType::Dungeon => DARKPURPLE, // Dungeon: dark purple
            };
            
            // Draw tile rectangle background
            draw_rectangle(screen_x, screen_y, tile_size, tile_size, color);
            
            // Draw tile's ASCII character
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
    
    // Draw items on map
    for ((x, y), item) in &game.current_map.items {
        // Calculate item's screen position
        let screen_x = start_x + (*x - game.camera_x) as f32 * tile_size;
        let screen_y = start_y + (*y - game.camera_y) as f32 * tile_size;
        
        // Draw item character in yellow
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
    
    // Draw all NPCs
    for npc in &game.npcs {
        // Calculate NPC's screen position
        let screen_x = start_x + (npc.x - game.camera_x) as f32 * tile_size;
        let screen_y = start_y + (npc.y - game.camera_y) as f32 * tile_size;
        
        // Set color based on hostility: red for enemies, green for friendly
        let color = if npc.hostile { RED } else { GREEN };
        
        // Draw NPC character
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
    
    // Draw player character (represented by @ symbol)
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

/// Draw user interface (status bar, message log, control hints)
fn draw_ui(game: &Game, font: &Font) {
    // === Draw top status bar ===
    // Black background
    draw_rectangle(0.0, 0.0, screen_width(), 30.0, BLACK);
    
    // Display player status info and current map
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
    
    // === Draw bottom message log ===
    let log_y = screen_height() - 120.0;
    // Semi-transparent black background
    draw_rectangle(0.0, log_y, screen_width(), 120.0, Color::new(0.0, 0.0, 0.0, 0.8));
    
    // Display most recent 5 messages
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
    
    // === Draw control hints ===
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

/// Draw inventory interface
fn draw_inventory(game: &Game, font: &Font) {
    // Calculate centered panel position
    let panel_w = 400.0;
    let panel_h = 300.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    
    // Draw panel background and border
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, WHITE);
    
    // Draw title
    draw_text_ex("INVENTORY", panel_x + 10.0, panel_y + 30.0, TextParams {
        font: Some(font),
        font_size: 24,
        color: YELLOW,
        ..Default::default()
    });
    
    // Display inventory contents
    if game.player.inventory.is_empty() {
        draw_text_ex("Empty", panel_x + 10.0, panel_y + 60.0, TextParams {
            font: Some(font),
            font_size: 20,
            color: GRAY,
            ..Default::default()
        });
    } else {
        // List all items
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
    
    // Draw close hint
    draw_text_ex("Press I to close", panel_x + 10.0, panel_y + panel_h - 20.0, TextParams {
        font: Some(font),
        font_size: 16,
        color: DARKGRAY,
        ..Default::default()
    });
}

/// Draw dialogue interface
/// Draw branching dialogue interface (West of Loathing style)
fn draw_dialogue(game: &Game, npc_idx: usize, node_idx: usize, selected: usize, font: &Font) {
    // Calculate dialogue box position (bottom of screen)
    let panel_w = 500.0;
    let panel_h = 200.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = screen_height() - panel_h - 50.0;

    // Draw dialogue box background and border
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, GREEN);

    // Get NPC data
    let npc = &game.npcs[npc_idx];

    // Get current dialogue node
    let node = &npc.dialogue[node_idx];

    // Display NPC name
    draw_text_ex(&npc.name, panel_x + 10.0, panel_y + 30.0, TextParams {
        font: Some(font),
        font_size: 22,
        color: GREEN,
        ..Default::default()
    });

    // Display current node text
    draw_text_ex(&node.text, panel_x + 10.0, panel_y + 60.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: WHITE,
        ..Default::default()
    });

    // Display all options, highlight selected option
    for (i, opt) in node.options.iter().enumerate() {
        let y = panel_y + 100.0 + i as f32 * 28.0;
        let color = if i == selected { YELLOW } else { GRAY };
        let prefix = if i == selected { "> " } else { "  " };
        draw_text_ex(&format!("{}{}", prefix, opt.text), panel_x + 30.0, y, TextParams {
            font: Some(font),
            font_size: 18,
            color,
            ..Default::default()
        });
    }

    // Draw hint
    draw_text_ex("↑↓Select, Enter/Space Confirm, ESC Exit", panel_x + 10.0, panel_y + panel_h - 20.0, TextParams {
        font: Some(font),
        font_size: 16,
        color: DARKGRAY,
        ..Default::default()
    });
}

/// Draw combat interface
fn draw_combat(game: &Game, npc_idx: usize, font: &Font) {
    // Calculate centered combat panel position
    let panel_w = 500.0;
    let panel_h = 250.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    
    // Draw combat panel background and border (red border indicates combat)
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, BLACK);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, RED);
    
    // Get enemy data
    let npc = &game.npcs[npc_idx];
    
    // Display combat title
    draw_text_ex("COMBAT", panel_x + 10.0, panel_y + 30.0, TextParams {
        font: Some(font),
        font_size: 24,
        color: RED,
        ..Default::default()
    });
    
    // Display enemy information
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
    
    // Display player information
    draw_text_ex(&format!("Your HP: {}/{}", game.player.hp, game.player.max_hp), 
              panel_x + 10.0, panel_y + 110.0, TextParams {
        font: Some(font),
        font_size: 18,
        color: WHITE,
        ..Default::default()
    });
    
    // Display combat options
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

// ========== Main Loop ==========

/// Game main loop
/// macroquad::main macro handles window creation and event loop
#[macroquad::main("Fallout-style RPG")]
async fn main() {
    // Load custom font
    let font = load_ttf_font("assets/fonts/JetBrainsMonoNL-Regular.ttf")
        .await
        .expect("Failed to load font");
    
    // Create game instance
    let mut game = Game::new();

    // Game main loop - executes once per frame
    loop {
        // Clear screen to black
        clear_background(BLACK);

        // ========== Input Processing ==========
        // Handle different inputs based on current game state
        match game.state {
            // Playing state: handle movement and open inventory
            GameState::Playing => {
                // Move up: W key or up arrow
                if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                    game.move_player(0, -1);
                }
                // Move down: S key or down arrow
                if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                    game.move_player(0, 1);
                }
                // Move left: A key or left arrow
                if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                    game.move_player(-1, 0);
                }
                // Move right: D key or right arrow
                if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                    game.move_player(1, 0);
                }
                // Open inventory: I key
                if is_key_pressed(KeyCode::I) {
                    game.state = GameState::Inventory;
                }
                // Enter town/dungeon: Space key
                if is_key_pressed(KeyCode::Space) {
                    game.try_enter_location();
                }
                // Return to world map: ESC key
                if is_key_pressed(KeyCode::Escape) {
                    game.return_to_world_map();
                }
            }
            
            // Inventory state: handle closing inventory
            GameState::Inventory => {
                // I key or ESC key closes inventory
                if is_key_pressed(KeyCode::I) || is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                }
            }
            
            // Dialogue state: handle option selection and transitions
            GameState::Dialogue(npc_idx, node_idx, selected) => {
                let npc = &game.npcs[npc_idx];
                let node = &npc.dialogue[node_idx];
                let num_options = node.options.len();
                
                // Up/Down keys to select options
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if selected > 0 {
                        game.state = GameState::Dialogue(npc_idx, node_idx, selected - 1);
                    }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if selected + 1 < num_options {
                        game.state = GameState::Dialogue(npc_idx, node_idx, selected + 1);
                    }
                }
                
                // Space or Enter to confirm selection
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    if let Some(next) = node.options[selected].next_node {
                        // Jump to next node
                        game.state = GameState::Dialogue(npc_idx, next, 0);
                    } else {
                        // End dialogue
                        game.state = GameState::Playing;
                    }
                }
                
                // ESC key exits dialogue
                if is_key_pressed(KeyCode::Escape) {
                    game.state = GameState::Playing;
                }
            }
            
            // Combat state: handle combat options
            GameState::Combat(npc_idx) => {
                // Option 1: Attack
                if is_key_pressed(KeyCode::Key1) {
                    // Calculate damage
                    let damage = 15;
                    game.npcs[npc_idx].hp -= damage;
                    game.add_message(format!("You dealt {} damage!", damage));
                    
                    // Check if enemy is defeated
                    if game.npcs[npc_idx].hp <= 0 {
                        game.add_message(format!("{} defeated!", game.npcs[npc_idx].name));
                        game.npcs.remove(npc_idx);  // Remove enemy from game
                        game.state = GameState::Playing;
                    } else {
                        // Enemy counterattack
                        let enemy_damage = 10;
                        game.player.hp -= enemy_damage;
                        game.add_message(format!("Enemy dealt {} damage!", enemy_damage));
                    }
                }
                
                // Option 3: Run
                if is_key_pressed(KeyCode::Key3) {
                    game.add_message("You ran away!".to_string());
                    game.state = GameState::Playing;
                }
            }
        }
        
        // ========== Update Game State ==========
        // Update camera position to follow player
        game.update_camera();
        
        // ========== Rendering ==========
        // Draw main game interface (map, NPCs, player)
        draw_game(&game, &font);
        
        // Draw UI elements (status bar, message log)
        draw_ui(&game, &font);
        
        // Draw additional interfaces based on current state
        match game.state {
            GameState::Inventory => draw_inventory(&game, &font),         // Inventory interface
            GameState::Dialogue(npc_idx, node_idx, selected) => draw_dialogue(&game, npc_idx, node_idx, selected, &font), // Dialogue interface
            GameState::Combat(idx) => draw_combat(&game, idx, &font),     // Combat interface
            _ => {}  // Playing state doesn't need extra interfaces
        }
        
        // Wait for next frame (controls frame rate, handles system events)
        next_frame().await;
    }
}