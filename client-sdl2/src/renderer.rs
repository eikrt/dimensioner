use crate::bitmap::*;
use crate::math::{dist_f32_i32, lerp};
use crate::ui::*;
use crate::util::{ActionContent, ActionType, ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{
    Camera, Chunk, Coords_f32, Coords_i32, Entity, EntityType, Faction, HashableF32, Tile,
    CHUNK_SIZE, TILE_SIZE, WORLD_SIZE,
};
use crossbeam::channel::unbounded;
use lazy_static::lazy_static;
use rand::rngs::StdRng; // Standard RNG implementation
use rand::Rng;
use rand::SeedableRng; // For seeding
use sdl2::event::{Event, WindowEvent};
use sdl2::image::{self, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{DisplayMode, Window, WindowContext};
use std::collections::HashMap;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 640;
    pub static ref WINDOW_HEIGHT: u32 = 360;
    pub static ref DEFAULT_ZOOM: i32 = 1;
    pub static ref CAMERA_STEP: HashableF32 = HashableF32(32.0);
    static ref PROJ_PLANE_W: u32 = 160;
    static ref PROJ_PLANE_H: u32 = 90;
    static ref FOV: f32 = 3.14 / 2.0;
    static ref PP_DIST: f32 = (*PROJ_PLANE_W as f32 / 2.0) / (*FOV / 2.0).tan();
    static ref COL_WIDTH: f32 = (*FOV / *PROJ_PLANE_W as f32) as f32;
}
#[derive(Debug)]
struct Pixel {
    rect: Rect,
    color: Color,
}
impl Pixel {
    fn new(rect: Rect, color: Color) -> Pixel {
        Pixel {
            rect: rect,
            color: color,
        }
    }
}
struct InputBuffer {
    ang: f32,
    cang: f32,
    traj: f32,
    forward: bool,
    c_pressed: bool,
    r_pressed: bool,
    l_pressed: bool,
    v_pressed: bool,
    e_pressed: bool,
    space_pressed: bool,
}

struct TileCache<'a> {
    textures: HashMap<u64, Texture<'a>>, // Assuming u32 as tile type identifier
}
struct TextureCache<'a> {
    textures: HashMap<u64, Texture<'a>>,
}
impl<'a> TextureCache<'a> {
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
        tex_cache: &TextureCache,
        time: f32,
    ) -> &Texture<'a> {
        // Use the chunk index or a unique identifier as the key for caching

        let seed: u64 = 42; // Any fixed number as a seed
        let mut rng = StdRng::seed_from_u64(seed);
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

                        let mut texture = tex_cache.textures.get(&0).unwrap();
                        // Define the color based on tile.z or other properties
                        let mut color = (
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 10.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                        );
                        if tile.coords.z < 0 {
                            color = (0, 0, 255); // Example for specific z-coordinates
                            texture = tex_cache.textures.get(&25).unwrap();
                        }
                        tex_canvas.set_draw_color(Color::RGBA(color.0, color.1, color.2, 100));

                        if tile.coords.z < 0 {
                            let dest_rect = Rect::new(
                                x * *TILE_SIZE as i32
                                    + (((time as f32 / 1000.0).sin() * 8.0) as i32),
                                y * *TILE_SIZE as i32 / 2
                                    + (((time as f32 / 1000.0).sin() * 8.0) as i32),
                                *TILE_SIZE,
                                *TILE_SIZE,
                            );
                            let src_rect = Rect::new(0, 0, 16, 16);
                            //tex_canvas.copy(&texture, src_rect, dest_rect).unwrap();
                        }
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

                        /*for i in 0..2 {
                                        for j in 0..2 {
                            if rng.gen_range(0..8) != 1 {
                                continue;
                            }

                            tex_canvas.set_draw_color(Color::RGBA(
                                                color.0 + rng.gen_range(0..16),
                                                255,
                                                color.0 + rng.gen_range(0..16),
                                                100,
                            ));
                            // Calculate position on the chunk texture
                            let dest_rect = Rect::new(
                                                x * *TILE_SIZE as i32 + j * 4,
                                                y * *TILE_SIZE as i32 + i * 4,
                                                1,
                                                1,
                            );
                            let src_rect =
                                                Rect::new(rng.gen_range(0..16), rng.gen_range(0..16), 1, 1);
                            tex_canvas.copy(&texture, src_rect, dest_rect).unwrap();
                            let _ = tex_canvas.fill_rect(dest_rect);
                                        }
                        }*/
                        i += 1;
                    }
                })
                .unwrap();
            canvas
                .with_texture_canvas(&mut texture, |tex_canvas| {
                    let mut i = 0;
                    for tile in &chunk.tiles {
                        let x = (i % *CHUNK_SIZE) as i32;
                        let y = (i / *CHUNK_SIZE) as i32;
                        let texture_creator = tex_canvas.texture_creator();
                        let mut texture = tex_cache.textures.get(&0).unwrap();
                        // Define the color based on tile.z or other properties
                        let mut color = (
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 10.0) as f32 * 255.0) as u8,
                            (255.0 - (tile.coords.z as f32 / 0.0) as f32 * 255.0) as u8,
                        );
                        if tile.coords.z < 0 {
                            color = (0, 0, 255); // Example for specific z-coordinates
                            texture = tex_cache.textures.get(&25).unwrap();
                        }
                        tex_canvas.set_draw_color(Color::RGBA(color.0, color.1, color.2, 100));

                        if tile.coords.z < 0 {
                            let dest_rect = Rect::new(
                                x * *TILE_SIZE as i32,
                                y * *TILE_SIZE as i32,
                                *TILE_SIZE + (((time as f32).sin()) * 8.0) as u32,
                                *TILE_SIZE,
                            );
                            let src_rect = Rect::new(0, 0, 16, 16);
                            tex_canvas.copy(&texture, src_rect, dest_rect).unwrap();
                            let _ = tex_canvas.fill_rect(dest_rect);
                        }
                        i += 1;
                    }
                })
                .unwrap();

            texture
        })
    }
}
fn shoot_ray(
    ang: f32,
    coords: (f32, f32, f32),
    chunk: &Chunk,
    camera: &Camera,
) -> (Option<(f32, f32, f32)>, Option<Color>) {
    // Calculate new coordinates based on angle
    let new_coords = (coords.0 + ang.cos(), coords.1 + ang.sin(), coords.2 + 1.0);

    // Check bounds to terminate recursion
    if coords.0 < 0.0
        || coords.0 > *PROJ_PLANE_W as f32
        || coords.1 < 0.0
        || coords.1 > *PROJ_PLANE_H as f32
        || coords.2 < 0.0
        || coords.2 > 8.0
    {
        return (None, None);
    }

    // Collect transformed points from the chunk tiles
    let mut points = vec![];
    for tile in &chunk.tiles {
        // Calculate color based on tile's z-coordinate
        let color = if tile.coords.z < 0 {
            Color::RGB(0, 0, 255) // Specific color for negative z
        } else {
            Color::RGB(
                (255.0 - (tile.coords.z as f32 / 10.0) * 255.0).clamp(0.0, 255.0) as u8,
                (255.0 - (tile.coords.z as f32 / 10.0) * 255.0).clamp(0.0, 255.0) as u8,
                (255.0 - (tile.coords.z as f32 / 10.0) * 255.0).clamp(0.0, 255.0) as u8,
            )
        };

        // Transform tile coordinates to camera space
        points.push((
            (
                (tile.coords.x as f32) * camera.scale_x as f32
                    + camera.coords.x.as_f32() / *TILE_SIZE as f32,
                (tile.coords.y as f32) * camera.scale_y as f32
                    + camera.coords.y.as_f32() / *TILE_SIZE as f32,
                0 as f32,
            ),
            color,
        ));
    }

    // Check if the current coordinates hit any tile
    
    for (point_coords, point_color) in &points {
	println!("{:?}, {:?}", point_coords, coords);
        if (
            point_coords.0.floor(),
            point_coords.1.floor(),
            point_coords.2.floor(),
        ) == (coords.0.floor(), coords.1.floor(), coords.2.floor())
        {
            return (Some(*point_coords), Some(*point_color));
        }
    }

    // Recursive call to continue the ray
    shoot_ray(ang, new_coords, chunk, camera)
}
fn render_scene(pixels: &mut Vec<Pixel>, chunk: &Chunk, camera: &Camera) {
    let col_width_f32 = *COL_WIDTH;

    // Create a thread-safe vector to collect pixels
    let pixels_par: Vec<Pixel> = (0..*WINDOW_WIDTH)
        .into_iter()
        .flat_map(|i| {
            let mut current_ang_x = i as f32 * col_width_f32;
            (0..1)
                .into_iter()
                .map(move |j| {
                    let rect = Rect::new(i as i32, j as i32, 50, 360);
                    let mut color = Color::RGB(0, 0, 0);
                    let coords = shoot_ray(current_ang_x, (i as f32, 0 as f32, 0.0), chunk, camera);
                    if coords == (None, None) {
                        color = Color::RGB(20, 20, 20);
                    } else {
                        let c = coords.0.unwrap();
                        let dist_from_viewer =
                            ((c.0 - 0.0).powf(2.0) + (c.1 - 0.0).powf(2.0) + (c.2 - 0.0).powf(2.0))
                                .sqrt();
                        color = coords.1.unwrap_or_else(|| Color::RGB(50,50,50));
                    }
                    Pixel::new(rect, color)
                })
                .collect::<Vec<_>>() // Collect results for this column
        })
        .collect();

    // Append all pixels to the original pixels vector
    pixels.extend(pixels_par);
}
fn draw_filled_circle(canvas: &mut Canvas<Window>, cx: f32, cy: f32, radius: i32, color: Color) {
    canvas.set_draw_color(color);

    for y in -radius..=radius {
        let dx = (radius.pow(2) as f32 - y.pow(2) as f32).sqrt() as f32;
        canvas
            .draw_line(
                ((cx - dx) as i32, (cy + y as f32 / 2.0) as i32),
                ((cx + dx) as i32, (cy + y as f32 / 2.0) as i32),
            )
            .unwrap();
    }
}
fn render_world<'a>(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    chunk: &Chunk,
    tile_cache: &mut TileCache<'a>,
    tex_cache: &TextureCache<'a>,
    texture_creator: &'a TextureCreator<WindowContext>,
    time: f32,
) {
    let chunk_texture =
        tile_cache.get_or_create_texture(camera, chunk, canvas, texture_creator, tex_cache, time);

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

fn draw_fog_layer(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    chunk: &Chunk,   // Assuming you have a `Chunk` type
    camera: &Camera, // Assuming you have a `Camera` type
    player: &Entity,
    tile_size: u32,
    chunk_size: u32,
    fog_alpha: u8, // Base alpha for fog (e.g., 200 for light fog)
    time: f32,
) {
    for tile in &chunk.tiles {
        let tile_x = (tile.coords.x as i32) * tile_size as i32 * camera.scale_x as i32
            + camera.coords.x.as_i32();
        let tile_y = (tile.coords.y as i32) * tile_size as i32 * camera.scale_y as i32
            + camera.coords.y.as_i32();
        let fog_width = tile_size * camera.scale_x as u32;
        let fog_height = tile_size * camera.scale_y as u32;
        let fog_alpha: u8 = 10;

        // Add random variation to fog alpha
        let mut rng = rand::thread_rng();

        let alpha_variation: u8 = (dist_f32_i32(
            &player.coords,
            &Coords_i32::from((
                tile.coords.x * *TILE_SIZE as i32,
                tile.coords.y * *TILE_SIZE as i32,
                tile.coords.z,
            )),
        ) * 1)
            .try_into()
            .unwrap_or_else(|_| 255); // Optional noise for fog effect
                                      // Set fog color with transparency
        let c = time_to_color(time);
        canvas.set_draw_color(Color::RGBA(c.r, c.g, c.b, alpha_variation));
        // Draw fog rectangle
        let fog_rect = Rect::new(tile_x, tile_y, fog_width, fog_height);
        canvas.fill_rect(fog_rect).unwrap();
    }
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
        .opengl()
        .build()
        .unwrap();
    let display_index = 0; // Primary display is typically index 0
    let display_mode: DisplayMode = video_subsystem.desktop_display_mode(display_index).unwrap();

    let mut camera = Camera::new();
    camera.scale_x = (display_mode.w as u32 / *WINDOW_WIDTH) as f32;
    camera.scale_y = (display_mode.h as u32 / *WINDOW_HEIGHT) as f32;
    camera.coords.x =
        HashableF32((*TILE_SIZE * *CHUNK_SIZE * *WORLD_SIZE) as f32 * -0.5 * camera.scale_x);
    camera.coords.y =
        HashableF32((*TILE_SIZE * *CHUNK_SIZE * *WORLD_SIZE) as f32 * -0.5 * camera.scale_y);
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "fonts/VastShadow-Regular.ttf";
    let font = ttf_context.load_font(font_path, 14).unwrap();
    let mut canvas = window
        .into_canvas()
        .accelerated()
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
        cang: 0.0,
        traj: 0.0,
        forward: false,
        c_pressed: false,
        r_pressed: false,
        l_pressed: false,
        v_pressed: false,
        e_pressed: false,
        space_pressed: false,
    };
    let mut last_frame_time = Instant::now();
    let mut r: Option<Vec<RenderMsg>> = None;
    let mut current_chunks: Vec<Chunk> = vec![];
    let mut player: Option<Entity> = None;
    let mut ui_state_entities: HashMap<i32, Entity> = HashMap::new();
    let mut ui_state_tiles: HashMap<i32, Tile> = HashMap::new();

    tex_cache.textures.insert(
        0,
        texture_creator.load_texture("res/tiles/grass.png").unwrap(),
    );
    tex_cache.textures.insert(
        1,
        texture_creator
            .load_texture("res/characters/human.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        2,
        texture_creator
            .load_texture("res/characters/cannon.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        3,
        texture_creator
            .load_texture("res/misc/cauliflower.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        4,
        texture_creator.load_texture("res/misc/lily.png").unwrap(),
    );
    tex_cache.textures.insert(
        5,
        texture_creator.load_texture("res/misc/tulip.png").unwrap(),
    );
    tex_cache.textures.insert(
        6,
        texture_creator.load_texture("res/misc/stone.png").unwrap(),
    );
    tex_cache.textures.insert(
        7,
        texture_creator
            .load_texture("res/characters/shell.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        8,
        texture_creator
            .load_texture("res/effects/explosion.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        9,
        texture_creator
            .load_texture("res/buildings/crossroad.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        10,
        texture_creator
            .load_texture("res/buildings/landmine.png")
            .unwrap(),
    ); // landmine
    tex_cache.textures.insert(
        11,
        texture_creator
            .load_texture("res/buildings/road_u_d.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        12,
        texture_creator
            .load_texture("res/buildings/road_curve_l_r.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        13,
        texture_creator
            .load_texture("res/buildings/road_curve_r_l.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        14,
        texture_creator
            .load_texture("res/buildings/road_curve_u_r.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        15,
        texture_creator
            .load_texture("res/buildings/road_curve_u_l.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        16,
        texture_creator
            .load_texture("res/buildings/road_stop_l_r.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        17,
        texture_creator
            .load_texture("res/buildings/road_stop_r_l.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        18,
        texture_creator
            .load_texture("res/buildings/road_stop_u_d.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        19,
        texture_creator
            .load_texture("res/buildings/road_stop_r_l.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        20,
        texture_creator
            .load_texture("res/buildings/road_t.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        21,
        texture_creator
            .load_texture("res/buildings/road_t_2.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        22,
        texture_creator
            .load_texture("res/buildings/road_t_3.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        23,
        texture_creator
            .load_texture("res/buildings/road_t_4.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        24,
        texture_creator
            .load_texture("res/buildings/road.png")
            .unwrap(),
    );
    tex_cache.textures.insert(
        25,
        texture_creator.load_texture("res/tiles/water.png").unwrap(),
    );
    tex_cache.textures.insert(
        26,
        texture_creator
            .load_texture("res/vehicles/car.png")
            .unwrap(),
    );
    let mut time = 0.0;
    'main: loop {
        let mouse_util = event_pump.mouse_state();
        let (m_x, m_y) = (mouse_util.x(), mouse_util.y());
        let (m_x_scaled, m_y_scaled) = (
            (mouse_util.x() - camera.coords.x.as_i32()) / camera.scale_x as i32 / *TILE_SIZE as i32,
            (mouse_util.y() - camera.coords.y.as_i32()) / camera.scale_y as i32 / *TILE_SIZE as i32,
        );
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time);
        let delta_seconds = delta_time.as_secs_f32();
        time += delta_seconds;
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
                        Keycode::Left => {
                            if ((camera.coords.x + *CAMERA_STEP).as_f32() < 0.0) {
                                camera.coords.x += *CAMERA_STEP
                            }
                        }
                        Keycode::Right => camera.coords.x -= *CAMERA_STEP,
                        Keycode::Up => {
                            if ((camera.coords.y + *CAMERA_STEP).as_f32() < 0.0) {
                                camera.coords.y += *CAMERA_STEP
                            }
                        }
                        Keycode::Down => {
                            if (camera.coords.y - *CAMERA_STEP).as_f32()
                                > (*WORLD_SIZE * *CHUNK_SIZE * *TILE_SIZE) as f32 * -1.0
                            {
                                camera.coords.y -= *CAMERA_STEP
                            }
                        }
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
                        Keycode::K => {
                            input_buffer.cang += 3.14 / 16.0;
                        }
                        Keycode::J => {
                            input_buffer.cang -= 3.14 / 16.0;
                        }
                        Keycode::T => {
                            input_buffer.traj += 0.1;
                        }
                        Keycode::Y => {
                            input_buffer.traj -= 0.1;
                        }
                        Keycode::Space => {
                            input_buffer.space_pressed = true;
                        }
                        Keycode::L => {
                            input_buffer.l_pressed = true;
                        }
                        Keycode::V => {
                            input_buffer.v_pressed = true;
                        }
                        Keycode::E => {
                            input_buffer.e_pressed = true;
                        }
                        _ => {}
                    }
                    trigger_refresh = true;
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if matches!(
                        key,
                        Keycode::W | Keycode::A | Keycode::S | Keycode::D | Keycode::Plus
                    ) {
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
            if chunk.coords.x * (camera.scale_x as i32)
                < (camera.ccoords.x) as i32 * camera.scale_x as i32
                || chunk.coords.y * (camera.scale_x as i32)
                    < (camera.ccoords.y) as i32 * camera.scale_y as i32
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
                    chunk.coords.x * *CHUNK_SIZE as i32 * camera.scale_x as i32
                        + camera.coords.x.as_i32(),
                    chunk.coords.y * *CHUNK_SIZE as i32 * camera.scale_y as i32
                        + camera.coords.y.as_i32(),
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
                time,
            );
            for m in &chunk.entities {
                if let Some(ref player) = player {
                    if m.index == player.index {
                        continue;
                    }
                }
                let mut color = ((0) as u8, 255 as u8, 0);
                color.0 = 0;
                color.1 = 255;
                color.2 = 255;
                canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));

                let dest_rect = Rect::new(
                    (m.coords.x.as_i32()) * camera.scale_y as i32 + camera.coords.x.as_i32(),
                    (m.coords.y.as_i32() - m.coords.z.as_i32()) * camera.scale_y as i32
                        + camera.coords.y.as_i32(),
                    *TILE_SIZE * camera.scale_x as u32,
                    *TILE_SIZE * camera.scale_y as u32,
                );
                let src_rect = Rect::new(0, 0, *TILE_SIZE as u32, *TILE_SIZE as u32);
                let mut i = 9;
                match m.etype {
                    EntityType::Human => i = 1,
                    EntityType::Cannon => i = 2,
                    EntityType::Cauliflower => i = 3,
                    EntityType::Lily => i = 4,
                    EntityType::Tulip => i = 5,
                    EntityType::Stone => i = 6,
                    EntityType::Shell => i = 7,
                    EntityType::Explosion => i = 8,
                    EntityType::Landmine => i = 10,
                    EntityType::Car => i = 26,
                    EntityType::Road => {
                        i = 9;
                        let entities_clone = chunk.entities.clone();
                        for (j, e) in chunk.entities.clone().into_iter().enumerate() {
                            /*if e.etype == EntityType::Road {
                            if Coords_f32::from((e.coords.x.as_f32() + *TILE_SIZE as f32, e.coords.y.as_f32(), e.coords.z.as_f32())) == m.coords {

                            }
                            else if Coords_f32::from((e.coords.x.as_f32() + *TILE_SIZE as f32,e.coords.y.as_f32(),e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32() - *TILE_SIZE as f32,e.coords.y.as_f32(),e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32(),e.coords.y.as_f32() + *TILE_SIZE as f32,e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32(),e.coords.y.as_f32() - *TILE_SIZE as f32,e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32() + *TILE_SIZE as f32,e.coords.y.as_f32() + *TILE_SIZE as f32,e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32() - *TILE_SIZE as f32,e.coords.y.as_f32() - *TILE_SIZE as f32,e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32() + *TILE_SIZE as f32,e.coords.y.as_f32() - *TILE_SIZE as f32,e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else if Coords_f32::from((e.coords.x.as_f32() - *TILE_SIZE as f32,e.coords.y.as_f32() + *TILE_SIZE as f32,e.coords.z.as_f32())) == m.coords {
                                i = 9;
                                break;
                            }
                            else {
                            }
                            }*/
                        }
                    }
                };
                if m.etype == EntityType::Shell {
                    draw_filled_circle(
                        &mut canvas,
                        (m.coords.x.as_f32() + 8.0) * camera.scale_x as f32
                            + camera.coords.x.as_f32(),
                        (14.0 + m.coords.y.as_f32()) * camera.scale_y as f32
                            + camera.coords.y.as_f32(),
                        8 * camera.scale_x as i32,
                        Color::RGBA(0, 0, 0, 70),
                    );
                }
                canvas
                    .copy(&tex_cache.textures.get(&i).unwrap(), src_rect, dest_rect)
                    .unwrap();
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
                let linked_entity = chunk
                    .entities
                    .iter()
                    .find(|e| e.linked_entity_id == m.index as u64);
                player = Some(m.clone());
                m.ang = HashableF32(input_buffer.ang);
                if input_buffer.forward {
                    m.vel.x = HashableF32(input_buffer.ang.sin() * 1.0) * HashableF32(128.0);
                    m.vel.y = HashableF32(-input_buffer.ang.cos() * 1.0) * HashableF32(128.0);
                    m.coords.x += HashableF32(m.vel.x.0 * delta_seconds);
                    m.coords.y += HashableF32(m.vel.y.0 * delta_seconds);
                }
                if input_buffer.c_pressed {
                    m.inventory.items[0].1 -= 100;
                    let _ = sx_client.send(ClientMsg::from(
                        m.clone(),
                        ActionContent::from(
                            ActionType::ConstructCannon,
                            HashableF32(input_buffer.cang),
                            HashableF32(input_buffer.traj),
                        ),
                    ));
                    input_buffer.c_pressed = false;
                } else if input_buffer.r_pressed {
                    m.inventory.items[0].1 -= 50;
                    let _ = sx_client.send(ClientMsg::from(
                        m.clone(),
                        ActionContent::from(
                            ActionType::ConstructRoad,
                            HashableF32(input_buffer.cang),
                            HashableF32(input_buffer.traj),
                        ),
                    ));
                    input_buffer.r_pressed = false;
                } else if input_buffer.space_pressed {
                    m.inventory.items[0].1 -= 25;
                    let _ = sx_client.send(ClientMsg::from(
                        m.clone(),
                        ActionContent::from(
                            ActionType::ConstructShell,
                            HashableF32(input_buffer.cang),
                            HashableF32(input_buffer.traj),
                        ),
                    ));
                    input_buffer.space_pressed = false;
                } else if input_buffer.l_pressed {
                    m.inventory.items[0].1 -= 50;
                    let _ = sx_client.send(ClientMsg::from(
                        m.clone(),
                        ActionContent::from(
                            ActionType::ConstructLandmine,
                            HashableF32(input_buffer.cang),
                            HashableF32(input_buffer.traj),
                        ),
                    ));
                } else if input_buffer.v_pressed {
                    m.inventory.items[0].1 -= 200;
                    let _ = sx_client.send(ClientMsg::from(
                        m.clone(),
                        ActionContent::from(
                            ActionType::ConstructCar,
                            HashableF32(input_buffer.cang),
                            HashableF32(input_buffer.traj),
                        ),
                    ));
                    input_buffer.v_pressed = false;
                } else if input_buffer.e_pressed {
                    m.inventory.items[0].1 -= 0;
                    let _ = sx_client.send(ClientMsg::from(
                        m.clone(),
                        ActionContent::from(
                            ActionType::Interact,
                            HashableF32(input_buffer.cang),
                            HashableF32(input_buffer.traj),
                        ),
                    ));
                    input_buffer.e_pressed = false;
                } else {
                    let _ = sx_client.send(ClientMsg::from(m.clone(), ActionContent::new()));
                }
                /*if !(m.ccoords == chunk.coords) {
                    canvas.set_draw_color(Color::RGBA(255,255,255,200));
                    canvas.fill_rect(Rect::new((chunk.tiles[0].coords.x as i32) * *TILE_SIZE as i32 * camera.scale_x as i32 + camera.coords.x.as_i32(), (chunk.tiles[0].coords.y * *TILE_SIZE as i32 ) * camera.scale_y as i32 + camera.coords.y.as_i32(), *CHUNK_SIZE * *TILE_SIZE * camera.scale_x as u32, *CHUNK_SIZE * *TILE_SIZE * camera.scale_y as u32));
                }*/
            }
            if let Some(mut m) = player.clone() {
                let mut color = ((0) as u8, 255 as u8, 0);
                color.0 = 255;
                color.1 = 0;
                color.2 = 0;
                canvas.set_draw_color(Color::RGB(color.0, color.1, color.2));
                let dest_rect = Rect::new(
                    m.coords.x.as_i32() * 1 as i32 * camera.scale_x as i32
                        + camera.coords.x.as_i32(),
                    m.coords.y.as_i32() * 1 as i32 * camera.scale_y as i32
                        + camera.coords.y.as_i32(),
                    *TILE_SIZE * camera.scale_x as u32,
                    *TILE_SIZE * camera.scale_y as u32,
                );
                let src_rect = Rect::new(0, 0, 16, 16);
                canvas
                    .copy(&tex_cache.textures.get(&1).unwrap(), src_rect, dest_rect)
                    .unwrap();
                draw_fog_layer(
                    &mut canvas,
                    chunk,
                    &camera,
                    &player.as_ref().unwrap().clone(),
                    *TILE_SIZE,
                    *CHUNK_SIZE,
                    220,
                    time,
                );
            }

           //  let mut pixels = Vec::new();
           // // render_scene(&mut pixels, chunk, &camera);
           //  for p in pixels.iter() {
           //      canvas.set_draw_color(p.color);
           //      let ratio_x = 160 / *WINDOW_WIDTH;
           //      let ratio_y = 144 / *WINDOW_HEIGHT;
           //      canvas
           //          .fill_rect(Rect::new(
           //              (p.rect.x as i32) as i32,
           //              (p.rect.y as i32) as i32,
           //              (p.rect.w) as u32,
           //              (p.rect.h) as u32,
           //          ))
           //          .unwrap();
           //  }
        }

        // "crosshair"
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
        canvas
            .fill_rect(Rect::new(
                (m_x / *TILE_SIZE as i32 / camera.scale_x as i32)
                    * *TILE_SIZE as i32
                    * camera.scale_x as i32,
                (m_y / *TILE_SIZE as i32 / camera.scale_y as i32)
                    * *TILE_SIZE as i32
                    * camera.scale_y as i32,
                *TILE_SIZE * camera.scale_x as u32,
                *TILE_SIZE * camera.scale_y as u32,
            ))
            .unwrap();

        for (k, mut v) in &mut ui_state_tiles {
            let text = &v.get_sheet();
            let color = Color::RGB(0, 0, 0); // White
            draw_text(
                &mut canvas,
                &camera,
                (m_x_scaled, m_y_scaled),
                &font,
                text,
                color,
                "ui/CharacterBox.toml",
                100,
                100,
            )
            .unwrap();
        }
        for (k, mut v) in &mut ui_state_entities {
            let text = &v.get_sheet();
            let color = Color::RGB(0, 0, 0); // White
            draw_text(
                &mut canvas,
                &camera,
                (m_x_scaled, m_y_scaled),
                &font,
                text,
                color,
                "ui/CharacterBox.toml",
                100 + 128,
                100,
            )
            .unwrap();
        }
        let coins = match player {
            Some(ref s) => s.inventory.get_coins(),
            None => 0,
        };
        let hp = match player {
            Some(ref s) => s.stats.health,
            None => 0,
        };
        let text = format!(
            "{}\n{}\n{}\n{}",
            input_buffer.cang, input_buffer.traj, coins, hp
        );
        let color = Color::RGB(0, 0, 0); // White
        draw_text(
            &mut canvas,
            &camera,
            (m_x_scaled, m_y_scaled),
            &font,
            &text,
            color,
            "ui/HUD.toml",
            0,
            (360.0 - 64.0) as i32,
        )
        .unwrap();
        canvas.present();
        canvas.clear();
        let _ = sx.send(MainMsg::from(camera.clone(), player.clone(), true));
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn time_to_color(time: f32) -> Color {
    let r = ((time / 32.0).sin() * 255.0) as u8;
    let g = ((time / 32.0).sin() * 255.0) as u8;
    let b = ((time / 32.0).sin() * 255.0) as u8;
    Color::RGBA(r, g, b, 255)
}
