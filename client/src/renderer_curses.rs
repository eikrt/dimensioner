use crate::math::{dist_f32_i32, lerp};
use crate::ui::*;
use crate::util::{ActionContent, ActionType, ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{
    Camera, Chunk, Class, Coords_f32, Coords_i32, DialogueTree, Entity, EntityType, Faction,
    HashableF32, Stats, Tile, TileType, CHUNK_SIZE, TILE_SIZE, WORLD_SIZE,
};
use crossbeam::channel::unbounded;
use lazy_static::lazy_static;
use pancurses::*;
use rand::rngs::StdRng; // Standard RNG implementation
use rand::Rng;
use rand::SeedableRng; // For seeding
use std::collections::HashMap;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

lazy_static! {
    pub static ref WINDOW_WIDTH: i32 = 80;
    pub static ref WINDOW_HEIGHT: i32 = 24;
    pub static ref HUD_WIDTH: i32 = 24;
    pub static ref HUD_HEIGHT: i32 = 80;
}

#[derive(Clone, Debug)]
pub struct VicinityBox {
    pub coords: Coords_f32,
}
impl VicinityBox {
    pub fn new() -> VicinityBox {
        VicinityBox {
            coords: Coords_f32::from((
                *WINDOW_WIDTH as f32 / 2.0 * *TILE_SIZE as f32,
                *WINDOW_HEIGHT as f32 / 2.0 * *TILE_SIZE as f32,
                0.0,
            )),
        }
    }
}

pub fn render_server(
    sx: &crossbeam::channel::Sender<MainMsg>,
    rx: &crossbeam::channel::Receiver<Vec<RenderMsg>>,
    sx_client: &crossbeam::channel::Sender<ClientMsg>,
    rx_client: &crossbeam::channel::Receiver<ClientMsg>,
) {
    let mut vicinity_box = VicinityBox::new();
    let mut camera = Camera::new();
    camera.coords.x = HashableF32(-40.0 * *TILE_SIZE as f32);
    camera.coords.y = HashableF32(-12.0 * *TILE_SIZE as f32);
    let mut last_frame_time = Instant::now();
    let mut r: Option<Vec<RenderMsg>> = None;
    let mut current_chunks: Vec<Chunk> = vec![];
    let mut player: Option<Entity> = None;
    let mut ui_state_entities: HashMap<i32, Entity> = HashMap::new();
    let mut ui_state_tiles: HashMap<i32, Tile> = HashMap::new();
    let mut time = 0.0;
    let window = initscr();
    window.keypad(true); // Enable function and arrow keys
    window.refresh();
    start_color(); // Enable color mode
    noecho(); // Don't echo input
    curs_set(0); // Hide the cursor

    // Initialize color pairs
    init_pair(1, COLOR_WHITE, COLOR_BLUE); // Blue tile
    init_pair(2, COLOR_WHITE, COLOR_BLACK); // Green tile
    init_pair(3, COLOR_BLACK, COLOR_WHITE); // Green tile
    init_pair(4, COLOR_BLACK, COLOR_YELLOW); // Green tile
    init_pair(5, COLOR_WHITE, COLOR_BLACK); // Green tile
    init_pair(6, COLOR_WHITE, COLOR_YELLOW); // Green tile
    init_pair(7, COLOR_WHITE, COLOR_GREEN); // Green tile

    let mut highlighted_entity: Option<Entity> = None;
    let mut highlighted_tile: Option<Tile> = None;
    let mut character_menu_show = false;
    let mut dialogue = false;
    let mut current_dialogue_tree: Option<DialogueTree> = None;
    let mut character_menu_nodes = vec!["Stats", "Skills", "Inventory", "Done"];
    loop {
        window.mvaddstr(0, 0, "<Game Title>");
        window.refresh();
        match window.getch() {
            Some(Input::Character('q')) => {
                // Quit the application
                break;
            }
            Some(Input::Character(c)) => {
                break;
            }
            Some(input) => {}
            None => (),
        }
    }
    window.clear();

    let classes = vec![
        Class::Detective,
        Class::Mailcarrier,
        Class::Chemist,
        Class::Businessman,
        Class::Engineer,
    ];
    let mut stats = Stats::new();
    let mut name = "".to_string();
    let mut chosen_class = Class::Detective;
    let mut selected_index = 0;
    let mut last_message: String = "You have embarked.".to_string();
    let mut settlement_message : String = "Nowhere".to_string();

    loop {
        window.clear();
        window.mvaddstr(0, 0, "Character Creation");
        window.mvaddstr(1, 0, "Select Your Class:");

        // Display classes and highlight the selected one
        for (i, class) in classes.iter().enumerate() {
            if i == selected_index {
                window.mvaddstr(2 + i as i32, 0, &format!("> {:?}", class));
                chosen_class = class.clone();
            } else {
                window.mvaddstr(2 + i as i32, 0, &format!("  {:?}", class));
            }
        }

        // Instruction to set the character name
        window.mvaddstr(
            2 + classes.len() as i32,
            0,
            "Press 'n' to enter your character name",
        );

        window.refresh();

        match window.getch() {
            Some(Input::Character('q')) => {
                // Quit the application
                break;
            }
            Some(Input::KeyUp) => {
                if selected_index > 0 {
                    selected_index -= 1;
                }
            }
            Some(Input::KeyDown) => {
                if selected_index < classes.len() - 1 {
                    selected_index += 1;
                }
            }
            Some(Input::Character('n')) => {
                // Enter the character name
                window.clear();
                window.mvaddstr(0, 0, "Enter your character's name:");
                window.refresh();

                let mut input_name = String::new();
                loop {
                    match window.getch() {
                        Some(Input::Character(c)) if c != '\n' => {
                            input_name.push(c);
                            window.mvaddstr(1, 0, &input_name);
                            window.refresh();
                        }
                        Some(Input::Character('\n')) => {
                            // Confirm name entry
                            if !input_name.is_empty() {
                                name = input_name.clone();
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Some(Input::Character('\n')) => {
                // Confirm the selection
                window.clear();
                stats = Stats::gen_from_class(&chosen_class);
                if name.is_empty() {
                    name = "John Doe".to_string(); // Default name if none provided
                }
                window.mvaddstr(0, 0, &name);
                window.mvaddstr(1, 0, format!("{:?}", &chosen_class));
                window.mvaddstr(2, 0, stats.stat_sheet_hard());
                window.refresh();
                window.getch(); // Wait for user input before exiting
                window.clear();
                window.mvaddstr(0, 0, &name);
                window.mvaddstr(1, 0, format!("{:?}", &chosen_class));
                window.mvaddstr(2, 0, stats.stat_sheet_soft());
                window.getch(); // Wait for user input before exiting
                selected_index = 0;
                break;
            }
            _ => {}
        }
    }
    window.nodelay(true);
    'main: loop {
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time);
        let delta_seconds = delta_time.as_secs_f32();
        time += delta_seconds;
        last_frame_time = now;
        camera.tick();
        window.refresh();
        if let Ok(rec) = rx.recv() {
            r = Some(rec);
        }
        for message in &r {
            for c in message {
                current_chunks.retain(|c2| c2.hash != c.chunk.hash);
            }
        }
        for message in &r {
            for c in message {
                // Remove any chunk with the same hash from current_chunks
                if let Some(pos) = current_chunks
                    .iter()
                    .position(|c2| c.chunk.index == c2.index)
                {
                    current_chunks.remove(pos);
                }
                // Add the new chunk to current_chunks
                current_chunks.push(c.chunk.clone());
            }
        }
        for chunk in &current_chunks {
            for (i, t) in chunk.tiles.iter().enumerate() {
                let mut s = " ";
                match t.ttype {
                    TileType::Water => {
                        window.attron(COLOR_PAIR(1));
                        s = "~";
                    }

                    TileType::Sand => {
                        window.attron(COLOR_PAIR(6));
                        s = "~";
                    }
                    TileType::Grass => {
                        window.attron(COLOR_PAIR(7));
                        s = ".";
                    }
                    TileType::StoneSand => {
                        window.attron(COLOR_PAIR(6));
                        s = ":";
                    }
                    TileType::FarmLand => {
                        window.attron(COLOR_PAIR(6));
                        s = ":";
                    }
                    TileType::WetLand => {
                        window.attron(COLOR_PAIR(2));
                        s = "~";
                    }
                    TileType::Asphalt => {
                        window.attron(COLOR_PAIR(6));
                        s = "[";
                    }
                    TileType::Salt => {
                        window.attron(COLOR_PAIR(6));
                        s = "~";
                    }
                    TileType::Wood => {
                        window.attron(COLOR_PAIR(6));
                        s = "-";
                    }
                    TileType::Concrete => {
                        window.attron(COLOR_PAIR(3));
                        s = "-";
                    }
                    TileType::Granite => {
                        window.attron(COLOR_PAIR(6));
                        s = "8";
                    }
                }
                window.mvaddstr(
                    t.coords.y - camera.coords.y.as_i32() / *TILE_SIZE as i32,
                    (t.coords.x - camera.coords.x.as_i32() / *TILE_SIZE as i32),
                    s,
                );

                if t.coords.x
                    == vicinity_box.coords.x.as_i32() / *TILE_SIZE as i32
                        + camera.coords.x.as_i32() / *TILE_SIZE as i32
                    && t.coords.y
                        == vicinity_box.coords.y.as_i32() / *TILE_SIZE as i32
                            + camera.coords.y.as_i32() / *TILE_SIZE as i32
                {
                    highlighted_tile = Some(t.clone());
                }
            }
            for e in &chunk.entities {
                if let Some(ref player) = player {
                    if e.index == player.index {
			if player.ccoords == chunk.coords {
			    settlement_message = chunk.settlement.as_ref().unwrap().name.clone();
			}
                        continue;
                    }
                }
                window.attron(COLOR_PAIR(3));
                window.mvaddstr(
                    e.coords.y.as_i32() / *TILE_SIZE as i32
                        - camera.coords.y.as_i32() / *TILE_SIZE as i32,
                    e.coords.x.as_i32() / *TILE_SIZE as i32
                        - camera.coords.x.as_i32() / *TILE_SIZE as i32,
                    format!("{:?}", e.etype)[0..1].to_string(),
                );
                if e.coords.x.as_i32() / *TILE_SIZE as i32
                    == vicinity_box.coords.x.as_i32() / *TILE_SIZE as i32
                        + camera.coords.x.as_i32() / *TILE_SIZE as i32
                    && e.coords.y.as_i32() / *TILE_SIZE as i32
                        == vicinity_box.coords.y.as_i32() / *TILE_SIZE as i32
                            + camera.coords.y.as_i32() / *TILE_SIZE as i32
                {
                    highlighted_entity = Some(e.clone());
                }
            }
        }

        for i in 0..(*HUD_HEIGHT) {
            for j in 0..(*HUD_WIDTH) {
                window.attron(COLOR_PAIR(5));
                window.mvaddch(i, j, ' ');
                if j == *HUD_WIDTH - 1 {
                    window.mvaddch(i, j, '|');
                }
            }
        }

        if let Some(ref mut highlighted_entity) = highlighted_entity {
            window.attron(COLOR_PAIR(5));
            for (i, s) in highlighted_entity.get_sheet().into_iter().enumerate() {
                window.mvaddstr(i as i32, 0, format!("{}", s));
            }
        }
        if let Some(ref mut highlighted_tile) = highlighted_tile {
            window.attron(COLOR_PAIR(5));
            window.mvaddstr(4, 0, format!("{}", &highlighted_tile.get_sheet()[0]));
        }
        for i in (0)..(*HUD_WIDTH) {
            window.mvaddstr(*WINDOW_HEIGHT / 2, i, "–");
        }
        if let Some(ref mut player) = player {
            window.mvaddstr(*WINDOW_HEIGHT / 2 + 1, 0, &player.name);
            window.mvaddstr(*WINDOW_HEIGHT / 2 + 2, 0, format!("{:?}", player.class));
            window.mvaddstr(*WINDOW_HEIGHT / 2 + 3, 0, format!("{:?}", player.etype));
            window.mvaddstr(
                *WINDOW_HEIGHT / 2 + 4,
                0,
                format!("HP: {:?}", player.stats.health),
            );
            window.mvaddstr(
                *WINDOW_HEIGHT / 2 + 5,
                0,
                format!("XP: {:?}", player.experience),
            );
            window.mvaddstr(
                *WINDOW_HEIGHT / 2 + 6,
                0,
                format!("Level: {:?}", player.level),
            );
        }

        window.mvaddstr(*WINDOW_HEIGHT / 2 - 1, 0, &last_message);
        window.mvaddstr(0, *WINDOW_WIDTH / 2, &settlement_message);
        window.mvaddstr(*WINDOW_HEIGHT - 4, 0, get_time_as_string());
        if let Ok(rm) = rx_client.recv() {
            window.refresh();
            let mut m = rm.player.clone();
            window.attron(COLOR_PAIR(3));
            window.mvaddstr(
                m.coords.y.as_i32() / *TILE_SIZE as i32
                    - camera.coords.y.as_i32() / *TILE_SIZE as i32,
                m.coords.x.as_i32() / *TILE_SIZE as i32
                    - camera.coords.x.as_i32() / *TILE_SIZE as i32,
                "@",
            );
            window.mvaddstr(
                vicinity_box.coords.y.as_i32() / *TILE_SIZE as i32,
                vicinity_box.coords.x.as_i32() / *TILE_SIZE as i32,
                "X",
            );
            player = Some(m.clone());
            if let Some(ref mut player) = player {
                player.stats = stats.clone();
                player.class = chosen_class.clone();
                player.name = name.clone();
                camera.coords.x = HashableF32(lerp(
                    camera.coords.x.as_f32() - *WINDOW_WIDTH as f32,
                    player.coords.x.as_f32(),
                    0.1,
                ));
                camera.coords.y = HashableF32(lerp(
                    camera.coords.y.as_f32() - *WINDOW_HEIGHT as f32,
                    player.coords.y.as_f32(),
                    0.1,
                ));
            }

            match window.getch() {
                Some(Input::Character('q')) => {
                    // Quit the application
                    endwin();
                    break;
                }
                Some(Input::Character(c)) => {
                    if c == 'w' {
                        m.coords.y -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'a' {
                        m.coords.x -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 's' {
                        m.coords.y += HashableF32(*TILE_SIZE as f32);
                    } else if c == 'd' {
                        m.coords.x += HashableF32(*TILE_SIZE as f32);
                    } else if c == 'h' {
                        vicinity_box.coords.x -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'j' {
                        vicinity_box.coords.y -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'k' {
                        vicinity_box.coords.y += HashableF32(*TILE_SIZE as f32);
                    } else if c == 'l' {
                        vicinity_box.coords.x += HashableF32(*TILE_SIZE as f32);
                    } else if c == 'm' {
                        character_menu_show = true;
                    } else if c == 'e' {
                        if let Some(ref mut highlighted_entity) = highlighted_entity {
                            current_dialogue_tree = highlighted_entity.dialogue.clone();
                            dialogue = true;
                        }
                    }
                }
                Some(input) => {}
                None => (),
            }
            if let Some(ref player) = player {
                if dialogue {
                    window.nodelay(false);
                    if let Some(ref current_dialogue_tree) = current_dialogue_tree {
                        parse_dialogue(&window, player, &mut stats, &current_dialogue_tree);
                    }
                    dialogue = false;
                }
                if character_menu_show {
                    window.nodelay(false);
                    loop {
                        window.clear();
                        window.mvaddstr(0, 0, "Character Menu");

                        for (i, n) in character_menu_nodes.iter().enumerate() {
                            if i == selected_index {
                                window.mvaddstr(2 + i as i32, 0, &format!("> {}", n));
                            } else {
                                window.mvaddstr(2 + i as i32, 0, &format!("  {}", n));
                            }
                        }
                        window.refresh();
                        match window.getch() {
                            Some(Input::Character('q')) => {
                                break;
                            }
                            Some(Input::KeyUp) => {
                                if selected_index > 0 {
                                    selected_index -= 1;
                                }
                            }
                            Some(Input::KeyDown) => {
                                if selected_index < classes.len() - 1 {
                                    selected_index += 1;
                                }
                            }
                            Some(Input::Character('\n')) => {
                                // Confirm the selection
                                if selected_index == 0 {
                                    window.clear();
                                    stats = player.stats.clone();
                                    window.mvaddstr(0, 0, &name);
                                    window.mvaddstr(1, 0, format!("{:?}", &chosen_class));
                                    window.mvaddstr(2, 0, stats.stat_sheet_hard());
                                    window.refresh();
                                    window.getch(); // Wait for user input before exiting
                                    selected_index = 0;
                                } else if selected_index == 1 {
                                    window.clear();
                                    stats = player.stats.clone();
                                    window.mvaddstr(0, 0, &name);
                                    window.mvaddstr(1, 0, format!("{:?}", &chosen_class));
                                    window.mvaddstr(2, 0, stats.stat_sheet_soft());
                                    window.refresh();
                                    window.getch(); // Wait for user input before exiting
                                    selected_index = 0;
                                } else if selected_index == 2 {
                                    window.clear();
                                    window.mvaddstr(0, 0, "Inventory");
                                    window.mvaddstr(1, 0, format!("{:?}", player.inventory));
                                    window.refresh();
                                    window.getch(); // Wait for user input before exiting
                                    selected_index = 0;
                                } else {
                                    character_menu_show = false;
                                    window.nodelay(true);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let _ = sx_client.send(ClientMsg::from(m.clone(), ActionContent::new()));
        } else if let Some(ref player) = player {
            let _ = sx_client.send(ClientMsg::from(player.clone(), ActionContent::new()));
        }

        let _ = sx.send(MainMsg::from(camera.clone(), player.clone(), true));

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn parse_dialogue(window: &Window, player: &Entity, stats: &mut Stats, current_dialogue_tree: &DialogueTree) {
    let mut selected_index = 0;
    loop {
        window.clear();
        window.mvaddstr(0, 0, "Dialogue Menu");
        let req_stats = &current_dialogue_tree.answer.requirement_stats;
        if let Some(req_stats) = req_stats {
            if player.stats.botanist < req_stats.botanist {
                window.mvaddstr(1 as i32, 0, "Your skills are lacking for this task...");
                window.getch();
                return;
            }
	    else {
		stats.botanist += 1;
	    }
        }
        window.mvaddstr(1, 0, format!("{:?}", current_dialogue_tree.answer.content));
        for (i, n) in current_dialogue_tree.nodes.iter().enumerate() {
            if let Some(n) = n {
                if i == selected_index {
                    window.mvaddstr(4 + i as i32, 0, &format!("> {:?}", n.message.content));
                } else {
                    window.mvaddstr(4 + i as i32, 0, &format!("  {:?}", n.message.content));
                }
            }
        }
        window.refresh();

        match window.getch() {
            Some(Input::Character('q')) => {
                return;
            }
            Some(Input::KeyUp) => {
                if selected_index > 0 {
                    selected_index -= 1;
                }
            }
            Some(Input::KeyDown) => {
                if selected_index < current_dialogue_tree.nodes.len() {
                    selected_index += 1;
                }
            }
            Some(Input::Character('\n')) => {
                // Confirm the selection
                if current_dialogue_tree.nodes.len() > 0 {
                    let current_selection = &current_dialogue_tree.nodes[selected_index];
                    if let Some(current_selection) = current_selection {
                        window.clear();
                        window.refresh();
                        parse_dialogue(window, player, stats, current_selection);
                        window.getch(); // Wait for user input before exiting
                        selected_index = 0;
                        return;
                    } else {
                        selected_index = 0;
                    }
                } else {
                    window.nodelay(true);
                    return;
                }
            }
            _ => {}
        }
    }
}
fn get_time_as_string() -> String {
    return "1.1.2080".to_string();
}
