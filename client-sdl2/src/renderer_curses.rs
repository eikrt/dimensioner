use crate::math::{dist_f32_i32, lerp};
use crate::ui::*;
use crate::util::{ActionContent, ActionType, ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{
    Camera, Chunk, Coords_f32, Coords_i32, Entity, EntityType, Faction, HashableF32, Tile,
    CHUNK_SIZE, TILE_SIZE, WORLD_SIZE,
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
    let display_index = 0; // Primary display is typically index 0
    let mut camera = Camera::new();
    camera.coords.x = HashableF32(40.0 * *TILE_SIZE as f32);
    camera.coords.y = HashableF32(12.0 * *TILE_SIZE as f32);
    let mut last_frame_time = Instant::now();
    let mut r: Option<Vec<RenderMsg>> = None;
    let mut current_chunks: Vec<Chunk> = vec![];
    let mut player: Option<Entity> = None;
    let mut ui_state_entities: HashMap<i32, Entity> = HashMap::new();
    let mut ui_state_tiles: HashMap<i32, Tile> = HashMap::new();
    let mut time = 0.0;
    let window = initscr();
    window.nodelay(true);
    window.keypad(true); // Enable function and arrow keys
    window.refresh();
    start_color(); // Enable color mode
    noecho(); // Don't echo input
    curs_set(0); // Hide the cursor

    // Initialize color pairs
    init_pair(1, COLOR_WHITE, COLOR_BLUE); // Blue tile
    init_pair(2, COLOR_WHITE, COLOR_GREEN); // Green tile
    init_pair(3, COLOR_BLACK, COLOR_GREEN); // Green tile
    init_pair(4, COLOR_BLACK, COLOR_YELLOW); // Green tile
    init_pair(5, COLOR_WHITE, COLOR_BLACK); // Green tile

    let mut highlighted_entity: Option<Entity> = None;
    let mut highlighted_tile: Option<Tile> = None;
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
                if t.coords.z < 0 {
                    window.attron(COLOR_PAIR(1));
                } else {
                    window.attron(COLOR_PAIR(2));
                }
                window.mvaddstr(
                    t.coords.y + camera.coords.y.as_i32() / *TILE_SIZE as i32,
                    (t.coords.x + camera.coords.x.as_i32() / *TILE_SIZE as i32),
                    " ",
                );

                if t.coords.x
                    == vicinity_box.coords.x.as_i32() / *TILE_SIZE as i32
                        - camera.coords.x.as_i32() / *TILE_SIZE as i32
                    && t.coords.y
                        == vicinity_box.coords.y.as_i32() / *TILE_SIZE as i32
                            - camera.coords.y.as_i32() / *TILE_SIZE as i32
                {
                    highlighted_tile = Some(t.clone());
                }
            }
            for e in &chunk.entities {
                if let Some(ref player) = player {
                    if e.index == player.index {
                        continue;
                    }
                }
                window.attron(COLOR_PAIR(3));
                window.mvaddstr(
                    e.coords.y.as_i32() / *TILE_SIZE as i32
                        + camera.coords.y.as_i32() / *TILE_SIZE as i32,
                    e.coords.x.as_i32() / *TILE_SIZE as i32
                        + camera.coords.x.as_i32() / *TILE_SIZE as i32,
                    "e",
                );
                if e.coords.x.as_i32() == vicinity_box.coords.x.as_i32() - camera.coords.x.as_i32()
                    && e.coords.y.as_i32()
                        == vicinity_box.coords.y.as_i32() - camera.coords.y.as_i32()
                {
                    highlighted_entity = Some(e.clone());
                }
            }
        }

        for i in 0..(*HUD_HEIGHT) {
            for j in 0..(*HUD_WIDTH) {
                window.attron(COLOR_PAIR(5));
                window.mvaddch(i, j, ' ');
            }
        }
        for i in 0..(*HUD_HEIGHT) {
            for j in (*WINDOW_WIDTH - *HUD_WIDTH)..(*HUD_HEIGHT) {
                window.attron(COLOR_PAIR(5));
                window.mvaddch(i, j, ' ');
            }
        }
        if let Some(ref mut highlighted_entity) = highlighted_entity {
            window.attron(COLOR_PAIR(5));
            window.mvaddstr(0, 0, &highlighted_entity.get_sheet());
        }
        if let Some(ref mut highlighted_tile) = highlighted_tile {
            window.attron(COLOR_PAIR(5));
            window.mvaddstr(8, 0, format!("{:?}", &highlighted_tile.get_sheet()));
        }
        if let Ok(rm) = rx_client.recv() {
            let mut m = rm.player.clone();
            window.attron(COLOR_PAIR(3));
            window.mvaddstr(
                m.coords.y.as_i32() / *TILE_SIZE as i32
                    + camera.coords.y.as_i32() / *TILE_SIZE as i32,
                m.coords.x.as_i32() / *TILE_SIZE as i32
                    + camera.coords.x.as_i32() / *TILE_SIZE as i32,
                "@",
            );
            window.mvaddstr(
                vicinity_box.coords.y.as_i32() / *TILE_SIZE as i32,
                vicinity_box.coords.x.as_i32() / *TILE_SIZE as i32,
                "X",
            );
            player = Some(m.clone());

            match window.getch() {
                Some(Input::Character('q')) => {
                    // Quit the application
                    endwin();
                    break;
                }
                Some(Input::Character(c)) => {
                    if c == 'w' {
                        m.coords.y -= HashableF32(*TILE_SIZE as f32);
                        camera.coords.y += HashableF32(*TILE_SIZE as f32);
                    } else if c == 'a' {
                        m.coords.x -= HashableF32(*TILE_SIZE as f32);
                        camera.coords.x += HashableF32(*TILE_SIZE as f32);
                    } else if c == 's' {
                        m.coords.y += HashableF32(*TILE_SIZE as f32);
                        camera.coords.y -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'd' {
                        m.coords.x += HashableF32(*TILE_SIZE as f32);
                        camera.coords.x -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'h' {
                        vicinity_box.coords.x -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'j' {
                        vicinity_box.coords.y -= HashableF32(*TILE_SIZE as f32);
                    } else if c == 'k' {
                        vicinity_box.coords.y += HashableF32(*TILE_SIZE as f32);
                    } else if c == 'l' {
                        vicinity_box.coords.x += HashableF32(*TILE_SIZE as f32);
                    }
                }
                Some(input) => {}
                None => (),
            }
            let _ = sx_client.send(ClientMsg::from(m.clone(), ActionContent::new()));
        } else if let Some(ref player) = player {
            let _ = sx_client.send(ClientMsg::from(player.clone(), ActionContent::new()));
        }

        window.refresh();
        let _ = sx.send(MainMsg::from(camera.clone(), player.clone(), true));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
