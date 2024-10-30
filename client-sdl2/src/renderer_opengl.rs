use crate::util::{ClientMsg, MainMsg, RenderMsg};
use crate::worldgen::{Chunk, Coords, Entity, Faction, CHUNK_SIZE, TILE_SIZE};
use gl::types::*;
use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Instant;

lazy_static! {
    pub static ref WINDOW_WIDTH: u32 = 1240;
    pub static ref WINDOW_HEIGHT: u32 = 760;
    pub static ref DEFAULT_ZOOM: f32 = 1.0;
}

#[derive(Clone)]
pub struct Camera {
    pub coords: Coords,
    pub zoom: f32,
}
impl Camera {
    pub fn new() -> Camera {
        Camera {
            coords: Coords::new(),
            zoom: *DEFAULT_ZOOM,
        }
    }
}

fn init_opengl() {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }
}

fn create_shader_program() -> GLuint {
    // Vertex shader
    let vertex_shader = r#"
        #version 330 core
        layout (location = 0) in vec3 position;
        layout (location = 1) in float height;
        uniform mat4 projection;
        uniform mat4 view;
        out float vertexHeight;
        void main() {
            vertexHeight = height;
            gl_Position = projection * view * vec4(position.x, height, position.z, 1.0);
        }
    "#;
    let fragment_shader = r#"
        #version 330 core
        in float vertexHeight;
        out vec4 FragColor;
        void main() {
            FragColor = vec4(vertexHeight, vertexHeight * 0.5, 0.3, 1.0); // Map height to color
        }
    "#;

    // Compile shaders and link program
    let program = unsafe {
        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, &vertex_shader.as_ptr().cast(), std::ptr::null());
        gl::CompileShader(vs);

        let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, &fragment_shader.as_ptr().cast(), std::ptr::null());
        gl::CompileShader(fs);

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vs);
        gl::AttachShader(shader_program, fs);
        gl::LinkProgram(shader_program);

        gl::DeleteShader(vs);
        gl::DeleteShader(fs);

        shader_program
    };
    program
}

fn create_terrain_buffer(chunk: &Chunk) -> (GLuint, usize) {
    let mut vertices = Vec::new();

    for tile in &chunk.tiles {
        vertices.push(tile.coords.x as f32);
        vertices.push(tile.height as f32);
        vertices.push(tile.coords.y as f32);
    }

    let (mut vbo, mut vao) = (0, 0);
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as GLsizeiptr,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            4 * std::mem::size_of::<gl::types::GLfloat>() as GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }
    (vao, vertices.len() / 3)
}

pub fn render_server(
    sx: &crossbeam::channel::Sender<MainMsg>,
    rx: &crossbeam::channel::Receiver<Vec<RenderMsg>>,
    sx_c: &crossbeam::channel::Sender<ClientMsg>,
    rx_c: &crossbeam::channel::Receiver<ClientMsg>,
) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Baltia", *WINDOW_WIDTH, *WINDOW_HEIGHT)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

    init_opengl();
    let shader_program = create_shader_program();
    let mut camera = Camera::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_frame_time = Instant::now();

    'main: loop {
        let now = Instant::now();
        let delta_time = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;

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
                } => camera.zoom += 0.1,
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => camera.zoom -= 0.1,
                _ => {}
            }
        }

        if let Ok(messages) = rx.try_recv() {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::UseProgram(shader_program);
                for message in &messages {
                    let chunk = &message.chunk;
                    let (vao, count) = create_terrain_buffer(chunk);

                    gl::BindVertexArray(vao);
                    gl::DrawArrays(gl::TRIANGLES, 0, count as i32);
                }
            }
        }
        let _ = sx.send(MainMsg::from(camera.clone(), None, true));
        window.gl_swap_window();
    }
}
