use crate::math::{dist_f32_f32, dist_f32_i32};
use crate::util::{ActionType};
use lazy_static::lazy_static;
use noise::{NoiseFn, Perlin};
use rand::prelude::SliceRandom;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub, Div, AddAssign, SubAssign};
lazy_static! {
    pub static ref WORLD_SIZE: u32 = 4;
    pub static ref CHUNK_SIZE: u32 = 16;
    pub static ref TILE_SIZE: u32 = 1;
    pub static ref NOISE_SCALE: f64 = 64.0;
    pub static ref VICINITY_DIST: i32 = 4;
    pub static ref HUMAN_NAMES_F: Vec<String> = vec![
        "Kate".to_string(),
        "Elsa".to_string(),
        "Karen".to_string(),
        "Jade".to_string()
    ];
    pub static ref HUMAN_NAMES_M: Vec<String> = vec![
        "John".to_string(),
        "Jack".to_string(),
        "Jacques".to_string(),
        "Tom".to_string(),
        "Arnold".to_string()
    ];
    pub static ref GENDERS: Vec<Gender> = vec![Gender::Male, Gender::Female];
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct HashableF32(pub f32);

impl Hash for HashableF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Floor the value and cast to an integer before hashing
        let floored = self.0.floor() as i32;
        floored.hash(state);
    }
}// Implement AddAssign trait for FlooredF32
impl AddAssign for HashableF32{
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

// Implement SubAssign trait for FlooredF32
impl SubAssign for HashableF32 {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl Eq for HashableF32 {}

// Implement Add trait for HashableF32
impl Add for HashableF32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        HashableF32(self.0 + other.0)
    }
}

// Implement Sub trait for HashableF32
impl Sub for HashableF32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        HashableF32(self.0 - other.0)
    }
}

// Implement Div trait for HashableF32
impl Div for HashableF32 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        HashableF32(self.0 / other.0)
    }
}

// Implement other necessary traits for convenience
impl PartialEq for HashableF32{
    fn eq(&self, other: &Self) -> bool {
        self.0.floor() == other.0.floor()
    }
}
impl TryInto<i32> for HashableF32{
    type Error = &'static str;

    fn try_into(self) -> Result<i32, Self::Error> {
        let floored_value = self.0.floor();
        
        // Check if the floored value is within i32 range
        if floored_value < (i32::MIN as f32) || floored_value > (i32::MAX as f32) {
            Err("Value is out of range for i32")
        } else {
            Ok(floored_value as i32)
        }
    }
}
impl HashableF32 {

    pub fn sqrt(&self) -> Self {
        HashableF32(self.0.sqrt())
    }
    pub fn as_i32(&self) -> i32 {
        self.0.floor() as i32
    }
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}
#[derive(Clone, Debug, Hash)]
pub struct Camera {
    pub coords: Coords_f32,
    pub ccoords: Coords_i32,
    pub render_distance_w: i32,
    pub render_distance_h: i32,
    pub zoom: i32,
}
impl Camera {
    pub fn new() -> Camera {
        Camera {
            coords: Coords_f32::new(),
            ccoords: Coords_i32::new(),
            render_distance_w: 128 as i32,
            render_distance_h: 128 as i32,
            zoom: 1,
        }
    }
    pub fn tick(&mut self) {
        self.ccoords.x = (self.coords.x.0 as f32 / *CHUNK_SIZE as f32) as i32;
        self.ccoords.y = (self.coords.y.0 as f32 / *CHUNK_SIZE as f32) as i32;
    }
}
#[derive(Clone, Deserialize, Serialize, Debug, Hash)]
pub struct Tasks {
    build: (u8, bool),
    fight: (u8, bool),
    animal_husbandry: (u8, bool),
    industry: (u8, bool),
    farm: (u8, bool),
    oil_rig: (u8, bool),
}
impl Tasks {
    pub fn new() -> Tasks {
        Tasks {
            build: (1, true),
            fight: (0, true),
            animal_husbandry: (0, true),
            industry: (0, true),
            farm: (0, true),
            oil_rig: (0, true),
        }
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Hash)]
pub enum Gender {
    Male,
    Female,
    Other,
}
pub fn gen_human_name(faction: Faction, gender: &Gender) -> String {
    match gender {
        Gender::Male => HUMAN_NAMES_M
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string(),
        Gender::Female => HUMAN_NAMES_M
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string(),
        Gender::Other => HUMAN_NAMES_M
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string(),
    }
}
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug, Hash)]
pub enum Item {
    Bread,
    Coin,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Inventory {
    items: Vec<Item>,
}
impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            items: vec![] 
        }
    }
    pub fn get_coins(&self) -> i32 {
	return 0;
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Stats {
    health: i8,
    hunger: u8,
    strength: u8,
    intelligence: u8,
    agility: u8,
}
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Hash)]
pub enum Faction {
    Empty,
    Marine,
    Irregular,
    Worm,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Personality {
    aggression: u8,
}
impl Personality {
    pub fn new() -> Personality {
        Personality { aggression: 0 }
    }
    pub fn gen() -> Personality {
        let mut rng = rand::thread_rng();
        Personality {
            aggression: rng.gen_range(0..100),
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Alignment {
    pub faction: Faction,
    pub personality: Personality,
}
impl Alignment {
    pub fn new() -> Alignment {
        Alignment {
            faction: Faction::Empty,
            personality: Personality::gen(),
        }
    }
    pub fn from(faction: Faction) -> Alignment {
        Alignment {
            faction: faction,
            personality: Personality::gen(),
        }
    }
}
impl Stats {
    pub fn new() -> Stats {
        Stats {
            health: 100,
            hunger: 100,
            strength: 10,
            intelligence: 10,
            agility: 10,
        }
    }
    pub fn gen() -> Stats {
        let mut rng = rand::thread_rng();
        Stats {
            health: 100,
            hunger: 100,
            strength: rng.gen_range(0..10),
            intelligence: rng.gen_range(0..10),
            agility: rng.gen_range(0..10),
        }
    }
}
#[derive(Clone, Serialize, PartialEq, Deserialize, Debug, Hash)]
pub enum Status {
    Talking,
    Fighting,
    Idle,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub enum TileType {
    Grass,
    WoodenWall,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub enum EntityType {
    Human,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Coords_i32 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Coords_i32 {

    pub fn as_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
	bincode::serialize(self)
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Coords_f32 {
    pub x: HashableF32,
    pub y: HashableF32,
    pub z: HashableF32,
}
impl Coords_f32 {

    pub fn as_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
	bincode::serialize(self)
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Size {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Size {
    pub fn from(size: (i32, i32, i32)) -> Size {
        Size {
            x: size.0,
            y: size.1,
            z: size.2,
        }
    }
}
impl Coords_f32 {
    pub fn from(coords: (f32, f32, f32)) -> Coords_f32 {
        Coords_f32 {
            x: HashableF32(coords.0),
            y: HashableF32(coords.1),
            z: HashableF32(coords.2),
        }
    }
    pub fn new() -> Coords_f32 {
        Coords_f32 { x: HashableF32(0.0), y: HashableF32(0.0), z: HashableF32(0.0) }
    }
}
impl Coords_i32 {
    pub fn from(coords: (i32, i32, i32)) -> Coords_i32 {
        Coords_i32 {
            x: coords.0,
            y: coords.1,
            z: coords.2,
        }
    }
    pub fn new() -> Coords_i32 {
        Coords_i32 { x: 0, y: 0, z: 0 }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Entity {
    pub coords: Coords_f32,
    pub ccoords: Coords_i32,
    pub current_action: ActionType,
    pub vel: Coords_f32,
    pub ang: HashableF32,
    pub etype: EntityType,
    pub stats: Stats,
    pub status: Status,
    pub index: usize,
    pub alignment: Alignment,
    pub inventory: Inventory,
    pub name: String,
    pub gender: Gender,
    pub tasks: Tasks,
    pub current_world: usize,
}
impl Entity {
    pub fn new(index: usize) -> Entity {
        Entity {
            coords: Coords_f32::new(),
            ccoords: Coords_i32::new(),
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
	    current_action: ActionType::Empty,
            etype: EntityType::Human,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: index,
            name: "".to_string(),
            gender: Gender::Female,
            tasks: Tasks::new(),
	    current_world: 0,
        }
    }
    pub fn gen_player(id: usize, x:f32, y:f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x,y,z)),
            ccoords: Coords_i32::from(((x/ *CHUNK_SIZE as f32).floor() as i32, (y / *CHUNK_SIZE as f32).floor() as i32, (z / *CHUNK_SIZE as f32).floor() as i32)),
	    current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            etype: EntityType::Human,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "".to_string(),
            gender: Gender::Female,
            tasks: Tasks::new(),
	    current_world: 0,
        }
    }
    pub fn from(
        index: usize,
        coords: Coords_f32,
        vel: (f32, f32, f32),
        etype: EntityType,
        stats: Stats,
        alignment: Alignment,
        name: String,
        gender: Gender,
	current_world: usize,
    ) -> Entity {
        Entity {
            coords: coords.clone(),
            ccoords: Coords_i32::from(((coords.x / HashableF32(*CHUNK_SIZE as f32)).as_i32() , (coords.y  / HashableF32(*CHUNK_SIZE as f32)).as_i32(), (coords.z / HashableF32(*CHUNK_SIZE as f32)).as_i32())),
	    current_action: ActionType::Empty,
            etype: etype,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            stats: stats,
            status: Status::Idle,
            index: index,
            alignment: alignment,
            inventory: Inventory::new(),
            name: name,
            gender: gender,
            tasks: Tasks::new(),
	    current_world: current_world
        }
    }
    pub fn resolve(&mut self, step_increment: i32) {
        // movement

        // self.coords.x += step_increment as f32 * self.vel.0;
        // self.coords.y += step_increment as f32 * self.vel.1;
	self.ccoords.x = (self.coords.x / HashableF32(*CHUNK_SIZE as f32)).as_i32();
	self.ccoords.y = (self.coords.y / HashableF32(*CHUNK_SIZE as f32)).as_i32();
        if self.stats.hunger > 0 {
            self.stats.hunger -= 1;
        }
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if self.stats.hunger == 0 {
            if self.stats.health >= 0 {
                //self.stats.health -= 2;
            } else {
                //self.stats.health = 0;
            }
        }
        // resolve tasks
        //
        if self.tasks.build.1 {}
    }
    pub fn resolve_against(&mut self, other: &mut Entity, step_increment: i32) {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if dist_f32_f32(&self.coords, &other.coords) <= *VICINITY_DIST {
            if other.status == Status::Fighting {
                let dmg = other.stats.strength * roll;
                self.stats.health -= dmg as i8;
                if self.alignment.personality.aggression > 25 {
                    self.status = Status::Fighting;
                }
            }
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Tile {
    pub coords: Coords_i32,
    pub index: usize,
    pub size: Size,
    pub ttype: TileType,
    pub holds: Option<Entity>,
    pub designed: Option<TileType>,
}

impl Tile {
    pub fn from(
        coords: Coords_i32,
        index: usize,
        size: Size,
        ttype: TileType,
        holds: Option<Entity>,
    ) -> Tile {
        Tile {
            coords,
            index,
            size,
            ttype,
            holds,
            designed: None,
        }
    }
    pub fn as_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
	bincode::serialize(self)
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Chunk {
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub coords: Coords_i32,
    pub index: usize,
    pub hash: u64, 
}

impl Chunk {
    pub fn from(tiles: Vec<Tile>, entities: Vec<Entity>, coords: Coords_i32, index: usize, hash: u64) -> Chunk {
        Chunk {
            tiles,
            entities,
            coords,
            index,
	    hash,
        }
    }
    pub fn as_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
	bincode::serialize(self)
    }
    pub fn as_string(&self) -> Result<String, serde_json::Error> {
	serde_json::to_string(self)
    }
    pub fn new() -> Chunk {
        Chunk {
            tiles: vec![],
            entities: vec![],
            coords: Coords_i32::new(),
            index: 0,
	    hash: 0,
        }
    }
    pub fn resolve(&mut self, step_increment: i32) {
        let mut hasher = Sha256::new();
        hasher.update(bincode::serialize(&self.tiles).unwrap());
        hasher.update(bincode::serialize(&self.entities).unwrap());
        let result = hasher.finalize();
	self.hash = u64::from_le_bytes(result[0..8].try_into().expect("Failed to get 8 bytes"));
	for i in 0..step_increment {
            for _t in &mut self.tiles {}
            let mut entities_clone = self.entities.clone();
            for clone in &mut entities_clone {
                for entity in &mut self.entities {
                    entity.resolve_against(clone, step_increment);
                }
            }
            for entity in &mut self.entities {
                entity.resolve(step_increment);
            }
            self.entities = self
                .entities
                .iter()
                .filter(|e| e.stats.health > 0)
                .cloned()
                .collect();
        }
    }
    pub fn gen(&mut self, seed: u32) -> Chunk {
        let mut rng = rand::thread_rng();
        let mut tiles: Vec<Tile> = vec![];
        let mut entities: Vec<Entity> = vec![];
        let fac_perlin = Perlin::new(seed);
        let perlin = Perlin::new(seed);
        let perlin2 = Perlin::new(seed + 1);
        let perlin3 = Perlin::new(seed + 2);

        let mut discard_entities = false;
        let mut faction = &Faction::Empty;
        if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.0
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.1
        {
            faction = &Faction::Worm;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.1
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.2
        {
            faction = &Faction::Worm;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.2
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.3
        {
            faction = &Faction::Irregular;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.3
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.4
        {
            faction = &Faction::Irregular;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.4
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.5
        {
            faction = &Faction::Irregular;
        } else {
            discard_entities = true;
        }
        for c in 0..(*CHUNK_SIZE as i32 * *CHUNK_SIZE as i32) {
            let x = c % (*CHUNK_SIZE as i32) + self.coords.x as i32 * *CHUNK_SIZE as i32;
            let y = (c / *CHUNK_SIZE as i32) + self.coords.y as i32 * *CHUNK_SIZE as i32;
            let a = 2.0;
            let n1 = perlin.get([
                (x as f64) / *NOISE_SCALE + 0.1,
                (y as f64) / *NOISE_SCALE + 0.1,
            ]) * a;

            let n2 = perlin2.get([
                (x as f64) / *NOISE_SCALE * 2.0 + 0.1,
                (y as f64) / *NOISE_SCALE * 2.0 + 0.1,
            ]) * a
                / 8.0;

            let n3 = perlin3.get([
                (x as f64) / (*NOISE_SCALE * 8.0) + 0.1,
                (y as f64) / (*NOISE_SCALE * 8.0) + 0.1,
            ]) * a
                * -8.0;
            let height: i32 = (n1 + n2 + n3 + rng.gen_range(-1.0..1.0)) as i32;
            let gender = GENDERS.choose(&mut rand::thread_rng()).unwrap();
            if height > 0 && !discard_entities && rng.gen_range(0..32) == 1 {
                entities.push(Entity::from(
                    c as usize,
                    Coords_f32::from((x as f32, y as f32, height as f32)),
                    (0.0, 0.0, 0.0),
                    EntityType::Human,
                    Stats::gen(),
                    Alignment::from(faction.clone()),
                    gen_human_name(faction.clone(), gender),
                    gender.clone(),
		    0,
                ))
            }
            tiles.push(Tile::from(
                Coords_i32::from((x, y, height as i32)),
                c as usize,
                Size::from((*TILE_SIZE as i32, *TILE_SIZE as i32, *TILE_SIZE as i32)),
                TileType::Grass,
                None,
            ));
        }
        Chunk {
            tiles: tiles,
            entities: entities,
            coords: self.coords.clone(),
            index: self.index,
	    hash: 0,
        }
    }
    pub fn fetch_tile(&self, index: usize) -> &Tile {
        &self.tiles[index]
    }
    pub fn inquire_news(&self) -> News {
        let mut news = vec![];
        let mut coin_count = 0;
        self.entities
            .iter()
            .map(|e| coin_count += e.inventory.get_coins());

        if coin_count < 10 {
            news.push("absolute poorness in region x\n".to_string())
        }
        News::from(news)
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct News {
    pub newscast: Vec<String>,
}
impl News {
    pub fn new() -> News {
        News { newscast: vec![] }
    }
    pub fn from(newscast: Vec<String>) -> News {
        News { newscast: newscast }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct World {
    pub chunks: Vec<Chunk>,
}
impl World {
    pub fn from(chunks: Vec<Chunk>) -> World {
        World { chunks }
    }
    pub fn fetch_chunk_mut(&mut self, index: usize) -> &mut Chunk {
        &mut self.chunks[index]
    }
    pub fn fetch_chunk(&self, index: usize) -> &Chunk {
        &self.chunks[index]
    }
    pub fn fetch_chunk_x_y(&self, x: f32, y: f32) -> &Chunk {
	let x_int = x as i32;
	let y_int = y as i32;
	&self.chunks[(y_int * *WORLD_SIZE as i32 + x_int) as usize]
    }
    pub fn update_chunk_with_entity(&mut self, mut entity: Entity) {
        let x_int = entity.ccoords.x as i32;
        let y_int = entity.ccoords.y as i32;
	entity.ccoords.x = (entity.coords.x / HashableF32(*CHUNK_SIZE as f32)).as_i32();
	entity.ccoords.y = (entity.coords.y / HashableF32(*CHUNK_SIZE as f32)).as_i32();
        let chunk = &mut self.chunks[(y_int * *WORLD_SIZE as i32 + x_int) as usize];
        
        // Try to find an entity with the same ID
        if let Some(existing_entity) = chunk.entities.iter_mut().find(|e| e.index == entity.index) {
            // Update the existing entity
            *existing_entity = entity;
        } else {
            // Add the new entity
            chunk.entities.push(entity);
        }
    }    
    pub fn resolve(&mut self, step_increment: i32) {
        self.chunks
            .par_iter_mut()
            .for_each(|c| c.resolve(step_increment));
    }
    pub fn resolve_between(&mut self, step_increment: i32) {}
}
pub fn worldgen(seed: u32) -> World {
    let mut chunks: Vec<Chunk> = vec![];
    for c in 0..((*WORLD_SIZE * *WORLD_SIZE) as i32) {
        let x = (c % *WORLD_SIZE as i32) as f32;
        let y = (c / *WORLD_SIZE as i32) as f32;
        chunks.push(Chunk::from(
            vec![],
            vec![],
            Coords_i32::from((x as i32, y as i32, 0)),
            c as usize,
	    0,
        ));
    }
    chunks.par_iter_mut().for_each(|c| *c = c.gen(seed));
    let world = World::from(chunks);
    world
}
