use crate::bitmap::*;
use crate::util::{ActionType, ActionContent, ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{
    Camera, Chunk, Coords_f32, Coords_i32, Entity, Faction, HashableF32, Tile, WORLD_SIZE, CHUNK_SIZE,
    TILE_SIZE, EntityType
};
use crate::math::{lerp};
use crate::ui::*;
use lazy_static::lazy_static;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::{MouseButton};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext, DisplayMode};
use sdl2::image::{self, LoadTexture};
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::f64::consts::PI;
use crossbeam::channel::unbounded;
lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 640;
    pub static ref WINDOW_HEIGHT: u32 = 360;
    pub static ref DEFAULT_ZOOM: i32 = 1;
    pub static ref CAMERA_STEP: HashableF32 = HashableF32(32.0);
}
struct InputBuffer {
    ang: f32,
    traj: f32,
    cannon_ang: f32,
    forward: bool,
    c_pressed: bool,
    r_pressed: bool
}

struct TileCache<'a> {
    textures: HashMap<u64, Texture<'a>>, // Assuming u32 as tile type identifier
}
struct TextureCache<'a> {
    textures: HashMap<u64, Texture<'a>>,
}
impl <'a> TextureCache<'a> {
    fn new() -> Self {
        TextureCache {
            textures: HashMap::new(),
        }
    }
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
        tex_cache: &TextureCache 
    ) -> &Texture<'a> {
        // Use the chunk index or a unique identifier as the key for caching
        let chunk_id = chunk.hash; // Assuming `Chunk` has a unique ID field
	self.textures.clear();
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
			let texture_creator = tex_canvas.texture_creator();
			
			let texture = tex_cache.textures.get(&0).unwrap();
                        // Define the color based on tile.z or other properties
                        let mut color = (
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 10.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                        );
                        if tile.coords.z < 0 {
                            color = (0, 0, 255); // Example for specific z-coordinates
                        }
                        tex_canvas.set_draw_color(Color::RGBA(color.0, color.1, color.2, 100));
                        // Calculate position on the chunk texture
                        let dest_rect = Rect::new(
                            x * *TILE_SIZE as i32,
                            y * *TILE_SIZE as i32,
                            *TILE_SIZE,
                            *TILE_SIZE,
                        );
			let src_rect = Rect::new(0, 0, 16, 16);
			tex_canvas.copy(&texture, src_rect, dest_rect).unwrap();
                        let _ = tex_canvas.fill_rect(dest_rect);
                        i += 1;
                    }
                })
                .unwrap();

            texture
        })
    }
}
fn draw_filled_circle(canvas: &mut Canvas<Window>, cx: f32, cy: f32, radius: i32, color: Color) {
    canvas.set_draw_color(color);

    for y in -radius..=radius {
        let dx = (radius.pow(2) as f32 - y.pow(2) as f32).sqrt() as f32;
        canvas.draw_line(((cx - dx) as i32, (cy + y as f32 / 2.0) as i32), ((cx + dx) as i32, (cy + y as f32 / 2.0) as i32) ).unwrap();
    }
}
fn render_world<'a>(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    chunk: &Chunk,
    tile_cache: &mut TileCache<'a>,
    tex_cache: &TextureCache<'a>,
    texture_creator: &'a TextureCreator<WindowContext>,
) {
    let chunk_texture = tile_cache.get_or_create_texture(camera, chunk, canvas, texture_creator, tex_cache);

    // Set the destination position for the chunk based on camera position
    let dest_x = chunk.coords.x * *TILE_SIZE as i32 * *CHUNK_SIZE as i32 * camera.scale_x as i32
        + camera.coords.x.as_i32();
    let dest_y = chunk.coords.y * *TILE_SIZE as i32 * *CHUNK_SIZE as i32 * camera.scale_y as i32
        + camera.coords.y.as_i32();
    let dest_rect = Rect::new(
        dest_x,
        dest_y,
        *CHUNK_SIZE * *TILE_SIZE * camera.scale_x as u32,
        *CHUNK_SIZE * *TILE_SIZE * camera.scale_y as u32,
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
        .fullscreen_desktop()
        .build()
        .unwrap();
    let display_index = 0; // Primary display is typically index 0
    let display_mode: DisplayMode = video_subsystem.desktop_display_mode(display_index).unwrap();

    let mut camera = Camera::new();
    camera.scale_x = (display_mode.w as u32 / *WINDOW_WIDTH) as f32;
    camera.scale_y = (display_mode.h as u32 / *WINDOW_HEIGHT) as f32;
    camera.coords.x = HashableF32((*TILE_SIZE * *CHUNK_SIZE * *WORLD_SIZE / 2) as f32 * -1.0);
    camera.coords.y = HashableF32((*TILE_SIZE * *CHUNK_SIZE * *WORLD_SIZE / 2) as f32 * -1.0);
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "fonts/VastShadow-Regular.ttf";
    let font = ttf_context.load_font(font_path, 14).unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let mut tile_cache = TileCache::new();
    let mut tex_cache = TextureCache::new();
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    let _texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut factions = false;
    let mut news = false;
    let mut trigger_refresh = false;
    let mut input_buffer: InputBuffer = InputBuffer {
        ang: 0.0,
	traj: 0.0,
	cannon_ang: 0.0,
        forward: false,
	c_pressed: false,
	r_pressed: false,
    };
    let mut last_frame_time = Instant::now();
    let mut r: Option<Vec<RenderMsg>> = None;
    let mut current_chunks: Vec<Chunk> = vec![];
    let mut player: Option<Entity> = None;
    let mut ui_state_entities: HashMap<i32, Entity> = HashMap::new();
    let mut ui_state_tiles: HashMap<i32, Tile> = HashMap::new();
    tex_cache.textures.insert(0, texture_creator.load_texture("res/tiles/grass.png").unwrap());
    tex_cache.textures.insert(1, texture_creator.load_texture("res/characters/human.png").unwrap());
    tex_cache.textures.insert(2, texture_creator.load_texture("res/characters/cannon.png").unwrap());
    tex_cache.textures.insert(3, texture_creator.load_texture("res/misc/cauliflower.png").unwrap());
    tex_cache.textures.insert(4, texture_creator.load_texture("res/misc/lily.png").unwrap());
    tex_cache.textures.insert(5, texture_creator.load_texture("res/misc/tulip.png").unwrap());
    tex_cache.textures.insert(6, texture_creator.load_texture("res/misc/stone.png").unwrap());
    tex_cache.textures.insert(7, texture_creator.load_texture("res/characters/shell.png").unwrap());
    tex_cache.textures.insert(8, texture_creator.load_texture("res/effects/explosion.png").unwrap());
    tex_cache.textures.insert(9, texture_creator.load_texture("res/buildings/crossroad.png").unwrap());
    'main: loop {
	
	let mouse_util = event_pump.mouse_state();
	let (m_x, m_y) = (mouse_util.x(), mouse_util.y());
	let (m_x_scaled, m_y_scaled) = ( mouse_util.x() / camera.scale_x as i32 / *TILE_SIZE as i32, mouse_util.y() / camera.scale_y as i32 / *TILE_SIZE as i32 );
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
                current_chunks.retain(|c2| c2.hash != c.chunk.hash);
            }
        }
	for message in &r {
	    for c in message {
		// Remove any chunk with the same hash from current_chunks
		if let Some(pos) = current_chunks.iter().position(|c2| c.chunk.index == c2.index) {
		    current_chunks.remove(pos);
		}
		// Add the new chunk to current_chunks
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
                    camera.scale_x += 0.21;
                    camera.scale_y += 0.21;
                    tile_cache.textures.clear();
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    camera.scale_x -= 0.21;
                    camera.scale_y -= 0.21;
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
                        Keycode::C => {
			    input_buffer.c_pressed = true;
                        }
                        Keycode::R => {
			    input_buffer.r_pressed = true;
                        }
                        Keycode::X => {
			    ui_state_entities.clear();
			    ui_state_tiles.clear();
                        }
                        Keycode::M => {
			    input_buffer.cannon_ang += 0.1;
                        }
                        Keycode::N => {
			    input_buffer.cannon_ang -= 0.1;
                        }
                        Keycode::T => {
			    input_buffer.traj += 0.1;
                        }
                        Keycode::Y => {
			    input_buffer.traj -= 0.1;
                        }
                        _ => {}
                    }
                    trigger_refresh = true;
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if matches!(key, Keycode::W | Keycode::A | Keycode::S | Keycode::D | Keycode::Plus) {
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
            if chunk.coords.x * (camera.scale_x as i32) < (camera.ccoords.x) as i32 * camera.scale_x as i32
                || chunk.coords.y * (camera.scale_x as i32) < (camera.ccoords.y) as i32 * camera.scale_y as i32
                || chunk.coords.x * camera.scale_x as i32
                    > (camera.ccoords.x as i32 + *WINDOW_WIDTH as i32 * *CHUNK_SIZE as i32)
                        * camera.scale_x as i32
                || chunk.coords.y * camera.scale_y as i32
                    > (camera.ccoords.y as i32 + *WINDOW_HEIGHT as i32 * *CHUNK_SIZE as i32)
                        * camera.scale_y as i32
            {
                continue;
            }
	    
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
                    &Faction::Marine => {
                        canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
                    }
                    &Faction::Irregular => {
                        canvas.set_draw_color(Color::RGBA(0, 0, 255, 100));
                    }
                    &Faction::Worm => {
                        canvas.set_draw_color(Color::RGBA(255, 255, 0, 100));
                    }
                };
                let _ = canvas.fill_rect(Rect::new(
                    chunk.coords.x * *CHUNK_SIZE as i32 * camera.scale_x as i32 + camera.coords.x.as_i32(),
                    chunk.coords.y * *CHUNK_SIZE as i32 * camera.scale_y as i32 + camera.coords.y.as_i32(),
                    *CHUNK_SIZE * *TILE_SIZE * camera.scale_x as u32,
                    *CHUNK_SIZE * *TILE_SIZE * camera.scale_y as u32,
                ));
            }
            render_world(
                &mut canvas,
                &camera,
                &chunk,
                &mut tile_cache,
		&tex_cache,
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

                let dest_rect = Rect::new(
                    (m.coords.x.as_i32()) * camera.scale_y as i32 + camera.coords.x.as_i32(),
                    (m.coords.y.as_i32() - m.coords.z.as_i32()) * camera.scale_y as i32 + camera.coords.y.as_i32(),
                    *TILE_SIZE * camera.scale_x as u32,
                    *TILE_SIZE * camera.scale_y as u32,
                );
		let src_rect = Rect::new(0, 0, *TILE_SIZE as u32, *TILE_SIZE as u32);
		let mut i = 0;
		match m.etype {
		    EntityType::Human => {i = 1},
		    EntityType::Cannon => {i = 2},
		    EntityType::Cauliflower => {i = 3},
		    EntityType::Lily => {i = 4},
		    EntityType::Tulip => {i = 5},
		    EntityType::Stone => {i = 6},
		    EntityType::Shell => {i = 7},
		    EntityType::Explosion => {i = 8},
		    EntityType::Road => {i = 9},
		};
		//draw_filled_circle(&mut canvas, (m.coords.x.as_f32() + 8.0) * camera.scale_x as f32 + camera.coords.x.as_f32(), (14.0 + m.coords.y.as_f32()) * camera.scale_y as f32 + camera.coords.y.as_f32(), 8 * camera.scale_x as i32, Color::RGBA(0, 0, 0, 70));
		canvas.copy(&tex_cache.textures.get(&i).unwrap(), src_rect, dest_rect).unwrap();
		if m_x_scaled == (m.coords.x).as_i32() && m_y_scaled == (m.coords.y).as_i32() {
		      if mouse_util.is_mouse_button_pressed(MouseButton::Left) {
			  if !ui_state_entities.contains_key(&(m.index as i32)) {
			      ui_state_entities.insert((m.index as i32), m.clone());
			  }
			  
		      }
		} 
            }
	    for e in &chunk.tiles {
		if m_x_scaled == (e.coords.x) && m_y_scaled == e.coords.y {
		      if mouse_util.is_mouse_button_pressed(MouseButton::Left) {
			  if !ui_state_tiles.contains_key(&(e.index as i32)) {
			      ui_state_tiles.insert((e.index as i32), e.clone());
			  }
			  
		      }
		}
	    }
            if let Ok(rm) = rx_client.try_recv() {
                let mut m = rm.player.clone();

                player = Some(m.clone());
                m.ang = HashableF32(input_buffer.ang);
                if input_buffer.forward {
                    m.vel.x = HashableF32(input_buffer.ang.sin() * 1.0) * HashableF32(128.0);
                    m.vel.y = HashableF32(-input_buffer.ang.cos() * 1.0) * HashableF32(128.0);
                    m.coords.x += HashableF32(m.vel.x.0 * delta_seconds);
                    m.coords.y += HashableF32(m.vel.y.0 * delta_seconds);
                }
		if input_buffer.c_pressed {
                    let _ = sx_client.send(ClientMsg::from(m.clone(), ActionContent::from(ActionType::ConstructCannon, HashableF32(input_buffer.ang), HashableF32(input_buffer.traj))));
		    input_buffer.c_pressed = false;
		}
		else if input_buffer.r_pressed {
                    let _ = sx_client.send(ClientMsg::from(m.clone(), ActionContent::from(ActionType::ConstructRoad, HashableF32(input_buffer.ang), HashableF32(input_buffer.traj))));
		    input_buffer.r_pressed = false;
		}
		else {
                    let _ = sx_client.send(ClientMsg::from(m.clone(), ActionContent::new()));
		}
            }
            if let Some(mut m) = player.clone() {

                let mut color = ((0) as u8, 255 as u8, 0);
                color.0 = 255;
                color.1 = 0;
                color.2 = 0;
                canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                let dest_rect = Rect::new(
                    m.coords.x.as_i32() * 1 as i32 * camera.scale_x as i32 + camera.coords.x.as_i32(),
                    m.coords.y.as_i32() * 1 as i32 * camera.scale_y as i32 + camera.coords.y.as_i32(),
                    *TILE_SIZE * camera.scale_x as u32,
                    *TILE_SIZE * camera.scale_y as u32,
                );
		let src_rect = Rect::new(0, 0, 16, 16);
		canvas.copy(&tex_cache.textures.get(&1).unwrap(), src_rect, dest_rect).unwrap();
            }
        }

	canvas.set_draw_color(Color::RGBA(255,255,255,100));
	canvas.fill_rect(Rect::new((m_x / *TILE_SIZE as i32 / camera.scale_x as i32) * *TILE_SIZE as i32 * camera.scale_x as i32, (m_y / *TILE_SIZE as i32 / camera.scale_y as i32) * *TILE_SIZE as i32 * camera.scale_y as i32, *TILE_SIZE * camera.scale_x as u32, *TILE_SIZE * camera.scale_y as u32)).unwrap();
	
	for (k,mut v) in &mut ui_state_tiles {
	    let text = &v.get_sheet();
	    let color = Color::RGB(0, 0, 0); // White
	    draw_text(
		&mut canvas,
		(m_x_scaled, m_y_scaled),
		&font,
		text,
		color,
		"ui/CharacterBox.toml",
		100,
		100,
	    ).unwrap();
	}
	for (k,mut v) in &mut ui_state_entities {
	    let text = &v.get_sheet();
	    let color = Color::RGB(0, 0, 0); // White
	    draw_text(
		&mut canvas,
		(m_x_scaled, m_y_scaled),
		&font,
		text,
		color,
		"ui/CharacterBox.toml",
		100 + 512,
		100,
	    ).unwrap();
	}
	canvas.present();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let _ = sx.send(MainMsg::from(camera.clone(), player.clone(), true));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
