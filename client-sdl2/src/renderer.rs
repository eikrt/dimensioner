use crate::bitmap::*;
use crate::util::{ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{
    Camera, Chunk, Coords_f32, Coords_i32, Entity, Faction, HashableF32, Tile, CHUNK_SIZE,
    TILE_SIZE,
};
use lazy_static::lazy_static;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crossbeam::channel::unbounded;
lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 1240;
    pub static ref WINDOW_HEIGHT: u32 = 760;
    pub static ref DEFAULT_ZOOM: i32 = 1;
    pub static ref CAMERA_STEP: HashableF32 = HashableF32(32.0);
}
struct InputBuffer {
    ang: f32,
    forward: bool,
}

struct TileCache<'a> {
    textures: HashMap<u64, Texture<'a>>, // Assuming u32 as tile type identifier
}

impl<'a> TileCache<'a> {
    fn new() -> Self {
        TileCache {
            textures: HashMap::new(),
        }
    }

    fn get_or_create_texture(
        &mut self,
        camera: &Camera,
        chunk: &Chunk,
        canvas: &mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> &Texture<'a> {
        // Use the chunk index or a unique identifier as the key for caching
        let chunk_id = chunk.hash; // Assuming `Chunk` has a unique ID field
        self.textures.entry(chunk_id as u64).or_insert_with(|| {
            // Create a large texture to render the whole chunk
            let texture_width = *CHUNK_SIZE * *TILE_SIZE;
            let texture_height = *CHUNK_SIZE * *TILE_SIZE;
            let mut texture = texture_creator
                .create_texture_target(None, texture_width, texture_height)
                .unwrap();
            // Render all tiles in the chunk onto this texture
            canvas
                .with_texture_canvas(&mut texture, |tex_canvas| {
                    tex_canvas.clear(); // Clear to ensure no residual data
                    let mut i = 0;
                    for tile in &chunk.tiles {
                        let x = (i % *CHUNK_SIZE) as i32;
                        let y = (i / *CHUNK_SIZE) as i32;
                        // Define the color based on tile.z or other properties
                        let mut color = (
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 10.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                        );
                        if tile.coords.z < 0 {
                            color = (0, 0, 255); // Example for specific z-coordinates
                        }
                        tex_canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                        // Calculate position on the chunk texture
                        let dest_rect = Rect::new(
                            x * *TILE_SIZE as i32,
                            y * *TILE_SIZE as i32,
                            *TILE_SIZE,
                            *TILE_SIZE,
                        );
                        let _ = tex_canvas.fill_rect(dest_rect);
                        i += 1;
                    }
                })
                .unwrap();

            texture
        })
    }
}
fn render_world<'a>(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    chunk: &Chunk,
    tile_cache: &mut TileCache<'a>,
    texture_creator: &'a TextureCreator<WindowContext>,
) {
    let chunk_texture = tile_cache.get_or_create_texture(camera, chunk, canvas, texture_creator);

    // Set the destination position for the chunk based on camera position
    let dest_x = chunk.coords.x * *TILE_SIZE as i32 * *CHUNK_SIZE as i32 * camera.zoom
        + camera.coords.x.as_i32();
    let dest_y = chunk.coords.y * *TILE_SIZE as i32 * *CHUNK_SIZE as i32 * camera.zoom
        + camera.coords.y.as_i32();
    let dest_rect = Rect::new(
        dest_x,
        dest_y,
        *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
        *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
    );

    // Render the cached chunk texture
    let _ = canvas.copy(chunk_texture, None, dest_rect);
}

pub fn render_server(
    sx: &crossbeam::channel::Sender<MainMsg>,
    rx: &crossbeam::channel::Receiver<Vec<RenderMsg>>,
    sx_client: &crossbeam::channel::Sender<ClientMsg>,
    rx_client: &crossbeam::channel::Receiver<ClientMsg>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Dimensioner", *WINDOW_WIDTH, *WINDOW_HEIGHT)
        .position_centered()
        // .fullscreen_desktop()
        .build()
        .unwrap();
    let mut camera = Camera::new();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "fonts/VastShadow-Regular.ttf";
    let _font = ttf_context.load_font(font_path, 48).unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let mut tile_cache = TileCache::new();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    let _texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut factions = false;
    let mut news = false;
    let mut trigger_refresh = false;
    let mut input_buffer: InputBuffer = InputBuffer {
        ang: 0.0,
        forward: false,
    };
    let mut last_frame_time = Instant::now();
    let mut r: Option<Vec<RenderMsg>> = None;
    let mut current_chunks: Vec<Chunk> = vec![];
    let mut player: Option<Entity> = None;
    'main: loop {
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time);
        let delta_seconds = delta_time.as_secs_f32();
        last_frame_time = now;
        camera.tick();
        if let Ok(rec) = rx.try_recv() {
            r = Some(rec);
        }
        for message in &r {
            for c in message {
                if current_chunks.iter().any(|c2| c.chunk.hash != c2.hash)
                    || current_chunks.len() == 0
                {
                    current_chunks.push(c.chunk.clone());
                }
            }
        }
        for message in &r {
            for c in message {
                current_chunks.retain(|c2| c2.hash != c.chunk.hash);
                current_chunks.push(c.chunk.clone());
            }
        }
        let mut pressed_keys = HashSet::new();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,

                Event::KeyDown {
                    keycode: Some(Keycode::Plus),
                    ..
                } => {
                    camera.zoom += 1;
                    tile_cache.textures.clear();
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    camera.zoom -= 1;
                    tile_cache.textures.clear();
                }

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    match key {
                        Keycode::Left => camera.coords.x += *CAMERA_STEP,
                        Keycode::Right => camera.coords.x -= *CAMERA_STEP,
                        Keycode::Up => camera.coords.y += *CAMERA_STEP,
                        Keycode::Down => camera.coords.y -= *CAMERA_STEP,
                        Keycode::W | Keycode::A | Keycode::S | Keycode::D => {
                            pressed_keys.insert(key);
                        }
                        Keycode::F => factions = !factions,
                        Keycode::N => {
                            news = !news;
                            canvas.set_draw_color(Color::RGB(0, 0, 0));
                            canvas.clear();
                        }
                        _ => {}
                    }
                    trigger_refresh = true;
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if matches!(key, Keycode::W | Keycode::A | Keycode::S | Keycode::D) {
                        pressed_keys.remove(&key);
                    }
                    if pressed_keys.is_empty() {
                        input_buffer.forward = false;
                    }
                }

                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(width, height) => {
                        canvas
                            .window_mut()
                            .set_size(width as u32, height as u32)
                            .unwrap();
                        camera.render_distance_w = width;
                        camera.render_distance_h = height;
                        canvas.present();
                    }
                    _ => {}
                },

                _ => {}
            }
        }

        // Handle WASD input for movement direction and angle
        if !pressed_keys.is_empty() {
            input_buffer.forward = true;
            input_buffer.ang = match (
                pressed_keys.contains(&Keycode::W),
                pressed_keys.contains(&Keycode::A),
                pressed_keys.contains(&Keycode::S),
                pressed_keys.contains(&Keycode::D),
            ) {
                (true, false, false, false) => 0.0,              // W
                (false, true, false, false) => -3.14 / 2.0,      // A
                (false, false, true, false) => 3.14,             // S
                (false, false, false, true) => 3.14 / 2.0,       // D
                (true, true, false, false) => -3.14 / 4.0,       // WA
                (true, false, false, true) => 3.14 / 4.0,        // WD
                (false, true, true, false) => -3.14 * 3.0 / 4.0, // AS
                (false, false, true, true) => 3.14 * 3.0 / 4.0,  // SD
                _ => input_buffer.ang,                           // No change for invalid states
            };
        }

        for chunk in &current_chunks {
            if chunk.coords.x * camera.zoom < (camera.ccoords.x) as i32 * camera.zoom
                || chunk.coords.y * camera.zoom < (camera.ccoords.y) as i32 * camera.zoom
                || chunk.coords.x * camera.zoom
                    > (camera.ccoords.x as i32 + *WINDOW_WIDTH as i32 * *CHUNK_SIZE as i32)
                        * camera.zoom
                || chunk.coords.y * camera.zoom
                    > (camera.ccoords.y as i32 + *WINDOW_HEIGHT as i32 * *CHUNK_SIZE as i32)
                        * camera.zoom
            {
                continue;
            }
            // for m in &chunk.tiles {
            //     let mut color = (
            //         (255.0 - (1.0 * m.coords.z as f32 / 0.0) * 255.0) as u8,
            //         (255.0 - (1.0 * m.coords.z as f32 / 10.0) * 255.0) as u8,
            //         (255.0 - (1.0 * m.coords.z as f32 / 0.0) * 255.0) as u8,
            //     );
            //     if m.coords.z < 0 {
            //         color.0 = 0;
            //         color.1 = 0;
            //         color.2 = 255;
            //     }
            //     canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
            //     let _ = canvas.fill_rect(Rect::new(
            //         m.coords.x as i32 * *TILE_SIZE as i32 * camera.zoom
            //             + camera.coords.x.as_i32(),
            //         m.coords.y as i32 * *TILE_SIZE as i32 * camera.zoom
            //             + camera.coords.y.as_i32(),
            //         *TILE_SIZE * camera.zoom as u32,
            //         *TILE_SIZE * camera.zoom as u32,
            //     ));
            // }
            if factions {
                let counts: HashMap<Faction, usize> =
                    chunk
                        .entities
                        .clone()
                        .iter()
                        .fold(HashMap::new(), |mut acc, entity| {
                            *acc.entry(entity.clone().alignment.faction).or_insert(0) += 1;
                            acc
                        });
                let max_value = counts
                    .iter()
                    .max_by_key(|&(_, v)| v)
                    .map(|(k, _)| k)
                    .unwrap_or(&Faction::Empty);

                match max_value {
                    &Faction::Empty => {
                        canvas.set_draw_color(Color::RGBA(0, 0, 0, 100));
                    }
                    &Faction::Hiisi => {
                        canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
                    }
                    &Faction::Virumaa => {
                        canvas.set_draw_color(Color::RGBA(0, 0, 255, 100));
                    }
                    &Faction::Kalevala => {
                        canvas.set_draw_color(Color::RGBA(255, 255, 0, 100));
                    }
                    &Faction::Pohjola => {
                        canvas.set_draw_color(Color::RGBA(0, 0, 255, 100));
                    }
                    &Faction::Tapiola => {
                        canvas.set_draw_color(Color::RGBA(0, 255, 0, 100));
                    }
                    &Faction::Novgorod => {
                        canvas.set_draw_color(Color::RGBA(255, 0, 0, 100));
                    }
                };
                let _ = canvas.fill_rect(Rect::new(
                    chunk.coords.x * *CHUNK_SIZE as i32 * camera.zoom + camera.coords.x.as_i32(),
                    chunk.coords.y * *CHUNK_SIZE as i32 * camera.zoom + camera.coords.y.as_i32(),
                    *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
                    *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
                ));
            }
            render_world(
                &mut canvas,
                &camera,
                &chunk,
                &mut tile_cache,
                &texture_creator,
            );
            for m in &chunk.entities {
                if let Some(ref player) = player {
                    if m.index == player.index {
                        continue;
                    }
                }
                let mut color = ((0) as u8, 255 as u8, 0);
                color.0 = 255;
                color.1 = 0;
                color.2 = 0;
                canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                let _ = canvas.fill_rect(Rect::new(
                    m.coords.x.as_i32() * *TILE_SIZE as i32 * camera.zoom
                        + camera.coords.x.as_i32(),
                    m.coords.y.as_i32() * *TILE_SIZE as i32 * camera.zoom
                        + camera.coords.y.as_i32(),
                    *TILE_SIZE * camera.zoom as u32,
                    *TILE_SIZE * camera.zoom as u32,
                ));
            }
            if let Ok(rm) = rx_client.try_recv() {
                let mut m = rm.player.clone();
                player = Some(m.clone());
                m.ang = HashableF32(input_buffer.ang);
                if input_buffer.forward {
                    m.vel.x = HashableF32(input_buffer.ang.sin() * 1.0);
                    m.vel.y = HashableF32(-input_buffer.ang.cos() * 1.0);
                    m.coords.x += HashableF32(m.vel.x.0 * delta_seconds);
                    m.coords.y += HashableF32(m.vel.y.0 * delta_seconds);
                }
                let _ = sx_client.send(ClientMsg::from(m.clone()));
            }
            if let Some(mut m) = player.clone() {
                let mut color = ((0) as u8, 255 as u8, 0);
                color.0 = 255;
                color.1 = 0;
                color.2 = 0;
                canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                let _ = canvas.fill_rect(Rect::new(
                    m.coords.x.as_i32() * *TILE_SIZE as i32 * camera.zoom
                        + camera.coords.x.as_i32(),
                    m.coords.y.as_i32() * *TILE_SIZE as i32 * camera.zoom
                        + camera.coords.y.as_i32(),
                    *TILE_SIZE * camera.zoom as u32,
                    *TILE_SIZE * camera.zoom as u32,
                ));
            }
        }

        canvas.present();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let _ = sx.send(MainMsg::from(camera.clone(), player.clone(), true));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
