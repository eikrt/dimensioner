use crate::bitmap::*;
use crate::util::{ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{Entity, Camera, Chunk, Coords, Faction, CHUNK_SIZE, TILE_SIZE};
use std::collections::HashSet;
use lazy_static::lazy_static;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::time::{Duration, Instant};
lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 1240;
    pub static ref WINDOW_HEIGHT: u32 = 760;
    pub static ref DEFAULT_ZOOM: i32 = 1;
    pub static ref CAMERA_STEP: f32 = 32.0;
}
struct InputBuffer {
    ang: f32,
    forward: bool,
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
        .window("Baltia", *WINDOW_WIDTH, *WINDOW_HEIGHT)
        .position_centered()
        // .fullscreen_desktop()
        .build()
        .unwrap();
    let mut camera = Camera::new();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "fonts/VastShadow-Regular.ttf";
    let _font = ttf_context.load_font(font_path, 48).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
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
    let mut r = None;
    'main: loop {
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time);
        let delta_seconds = delta_time.as_secs_f32();
	let mut player: Option<Entity> = None;
        last_frame_time = now;
        camera.tick();
        if let Ok(rec) = rx.try_recv() {
            r = Some(rec);
        }
        match r {
            Some(ref r) => {
                for message in r {
                    let chunk = &message.chunk;
                    if chunk.coords.x as i32 * camera.zoom < (camera.ccoords.x) as i32 * camera.zoom
                        || chunk.coords.y as i32 * camera.zoom
                            < (camera.ccoords.y) as i32 * camera.zoom
                        || chunk.coords.x as i32 * camera.zoom
                            > (camera.ccoords.x as i32 + *WINDOW_WIDTH as i32 * *CHUNK_SIZE as i32)
                                * camera.zoom
                        || chunk.coords.y as i32 * camera.zoom
                            > (camera.ccoords.y as i32 + *WINDOW_HEIGHT as i32 * *CHUNK_SIZE as i32)
                                * camera.zoom
                    {
                        continue;
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
                            } => camera.zoom += 1,

                            Event::KeyDown {
                                keycode: Some(Keycode::Minus),
                                ..
                            } => camera.zoom -= 1,

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
                                if matches!(key, Keycode::W | Keycode::A | Keycode::S | Keycode::D)
                                {
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
                            _ => input_buffer.ang, // No change for invalid states
                        };
                    }
                    if trigger_refresh {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                        canvas.clear();
                        trigger_refresh = false;
                    }
                    if news {
                        let mut row = 0;
                        let mut index = 0;
                        for (i, n) in message.news.newscast.iter().enumerate() {
                            let mut text = n;
                            let char_span = 8;
                            let row_span = 14;

                            for c in text.chars() {
                                let v = LETTERS.get(&c).unwrap().clone();

                                if c == '\n' {
                                    row += 1;
                                    index = -row;
                                    continue;
                                }
                                for (k2, v2) in v.map {
                                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                                    match v2 {
                                        '#' => {
                                            let _ = canvas.fill_rect(Rect::new(
                                                k2.0 * *TILE_SIZE as i32 * camera.zoom
                                                    + camera.coords.x as i32
                                                    + index * char_span,
                                                k2.1 * *TILE_SIZE as i32 * camera.zoom
                                                    + camera.coords.y as i32
                                                    + row * row_span
                                                    + 16
                                                    + i as i32 * row_span,
                                                *TILE_SIZE * camera.zoom as u32,
                                                *TILE_SIZE * camera.zoom as u32,
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                                index += 1;
                            }
                            row += 1;
                            index = 0;
                        }
                        continue;
                    }
                    for m in &chunk.tiles {
                        let mut color = (
                            (255.0 - (1.0 * m.height as f32 / 0.0) * 255.0) as u8,
                            (255.0 - (1.0 * m.height as f32 / 10.0) * 255.0) as u8,
                            (255.0 - (1.0 * m.height as f32 / 0.0) * 255.0) as u8,
                        );
                        if m.height < 0 {
                            color.0 = 0;
                            color.1 = 0;
                            color.2 = 255;
                        }
                        canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                        let _ = canvas.fill_rect(Rect::new(
                            m.coords.x as i32 * *TILE_SIZE as i32 * camera.zoom
                                + camera.coords.x as i32,
                            m.coords.y as i32 * *TILE_SIZE as i32 * camera.zoom
                                + camera.coords.y as i32,
                            *TILE_SIZE * camera.zoom as u32,
                            *TILE_SIZE * camera.zoom as u32,
                        ));
                    }
                    if let Ok(rm) = rx_client.recv() {
                        let mut m = rm.player.clone();
			player = Some(m.clone());
                        let mut color = ((0) as u8, 255 as u8, 0);
                        color.0 = 255;
                        color.1 = 0;
                        color.2 = 0;
                        canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                        let _ = canvas.fill_rect(Rect::new(
                            m.coords.x as i32 * *TILE_SIZE as i32 * camera.zoom
                                + camera.coords.x as i32,
                            m.coords.y as i32 * *TILE_SIZE as i32 * camera.zoom
                                + camera.coords.y as i32,
                            *TILE_SIZE * camera.zoom as u32,
                            *TILE_SIZE * camera.zoom as u32,
                        ));
                        m.ang = input_buffer.ang;
                        if input_buffer.forward {
                            m.vel.0 = input_buffer.ang.sin() * 1.0;
                            m.vel.1 = -input_buffer.ang.cos() * 1.0;
                            m.coords.x += m.vel.0 * delta_seconds;
                            m.coords.y += m.vel.1 * delta_seconds;
                        }
                        let _ = sx_client.send(ClientMsg::from(m.clone()));
                    }
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
                            m.coords.x as i32 * *TILE_SIZE as i32 * camera.zoom
                                + camera.coords.x as i32,
                            m.coords.y as i32 * *TILE_SIZE as i32 * camera.zoom
                                + camera.coords.y as i32,
                            *TILE_SIZE * camera.zoom as u32,
                            *TILE_SIZE * camera.zoom as u32,
                        ));
                    }
                    if factions {
                        let counts: HashMap<Faction, usize> = chunk.entities.clone().iter().fold(
                            HashMap::new(),
                            |mut acc, entity| {
                                *acc.entry(entity.clone().alignment.faction).or_insert(0) += 1;
                                acc
                            },
                        );
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
                            chunk.coords.x as i32 * *CHUNK_SIZE as i32 * camera.zoom
                                + camera.coords.x as i32,
                            chunk.coords.y as i32 * *CHUNK_SIZE as i32 * camera.zoom
                                + camera.coords.y as i32,
                            *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
                            *CHUNK_SIZE * *TILE_SIZE * camera.zoom as u32,
                        ));
                    }
                }
            }
            None => {}
        }
        canvas.present();
        let _ = sx.send(MainMsg::from(camera.clone(), player.clone(), true));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
