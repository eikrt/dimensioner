use serde::Deserialize;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;
use std::collections::HashMap;
use std::fs;
use crate::worldgen::Camera;
#[derive(Deserialize, Debug)]
struct CharacterBox {
    box_dimensions: HashMap<String, BoxDimensions>,
}

#[derive(Deserialize, Debug)]
struct BoxDimensions {
    width: u32,
    height: u32,
}

pub fn draw_text(
    canvas: &mut Canvas<Window>,
    camera: &Camera,
    m_coords: (i32,i32),
    font: &Font,
    text: &str,
    color: Color,
    character_box_path: &str,
    box_x: i32,
    box_y: i32,
) -> Result<(), String> {
    let box_x = box_x * camera.scale_x as i32;
    let box_y = box_y * camera.scale_y as i32;
    // Load and parse CharacterBox.toml
    let toml_str = fs::read_to_string(character_box_path)
        .map_err(|e| format!("Failed to read CharacterBox.toml: {}", e))?;
    let character_box: CharacterBox = toml::from_str(&toml_str)
        .map_err(|e| format!("Failed to parse CharacterBox.toml: {}", e))?;

    // Fetch the box dimensions
    let box_dims = character_box
        .box_dimensions
        .get("dimensions")
        .ok_or("Failed to find 'dimensions' in CharacterBox.toml")?;

    // Draw the brass-colored background
    let brass_color = Color::RGB(181, 166, 66); // Brass-like color
    canvas.set_draw_color(brass_color);
    let background_rect = Rect::new(box_x, box_y, box_dims.width * camera.scale_x as u32, box_dims.height * camera.scale_y as u32);
    canvas.fill_rect(background_rect)?;

    // Draw the boundaries (a black rectangle around the box)
    let border_color = Color::RGB(0, 0, 0); // Black
    canvas.set_draw_color(border_color);
    canvas.draw_rect(background_rect)?;

    // Variables to track the current position within the box
    let mut current_x = box_x;
    let mut current_y = box_y;

    // Split the text into lines using '\n'
    for line in text.split('\n') {
        // Render the current line to a surface
        let surface = font
            .render(line)
            .blended(color)
            .map_err(|e| format!("Failed to render line '{}': {}", line, e))?;

        // Convert the surface to a texture
        let texture_creator = canvas.texture_creator();
        let texture = surface
            .as_texture(&texture_creator)
            .map_err(|e| format!("Failed to create texture for line '{}': {}", line, e))?;

        // Calculate the target rectangle for the current line
        let target = Rect::new(current_x, current_y, 16 * line.len() as u32, 16);

        // Render the texture to the canvas
        canvas.copy(&texture, None, Some(target))?;

        // Move to the next line position
        current_y += 16;

        // Stop rendering if the text exceeds the box height
        if current_y + 16 > box_y + box_dims.height as i32 {
            break;
        }
    }

    Ok(())
}
