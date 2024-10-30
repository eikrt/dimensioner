use crate::worldgen::Coords;
pub fn dist(c1: &Coords, c2: &Coords) -> i32 {
    ((c1.x - c2.x + c1.y - c2.y) as f32).sqrt() as i32
}
