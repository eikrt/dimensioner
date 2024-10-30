use godot::prelude::*;
use godot::classes::Sprite2D;
use godot::classes::ISprite2D;
struct DimensionerExtension;
#[gdextension]
unsafe impl ExtensionLibrary for DimensionerExtension {}

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct Player {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>
}
#[godot_api]
impl ISprite2D for Player {
    fn init(base: Base<Sprite2D>) -> Self {
        godot_print!("Hello, world!"); // Prints to the Godot console
        
        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }
}

pub mod lang;
pub mod math;
pub mod util;
pub mod worldgen;
pub mod net;
