use crate::worldgen::{Coords_f32, Coords_i32, HashableF32};
pub fn dist_f32_i32(c1: &Coords_f32, c2: &Coords_i32) -> i32 {
    ((c1.x - HashableF32(c2.x as f32) + c1.y - HashableF32(c2.y as f32))).sqrt().0 as i32
}

pub fn dist_f32_f32(c1: &Coords_f32, c2: &Coords_f32) -> i32 {
    ((c1.x - c2.x + c1.y - c2.y)).sqrt().0 as i32
}
