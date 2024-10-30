use crate::math::dist;
use lazy_static::lazy_static;
use noise::{NoiseFn, Perlin};
use rand::prelude::SliceRandom;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
lazy_static! {
    pub static ref WORLD_SIZE: u32 = 8;
    pub static ref CHUNK_SIZE: u32 = 16;
    pub static ref TILE_SIZE: u32 = 1;
    pub static ref NOISE_SCALE: f64 = 64.0;
    pub static ref VICINITY_DIST: i32 = 4;
    pub static ref HUMAN_NAMES_F: Vec<String> = vec![
        "Kirsika".to_string(),
        "Markus".to_string(),
        "Annika".to_string(),
        "Maris".to_string()
    ];
    pub static ref HUMAN_NAMES_M: Vec<String> = vec![
        "Hans".to_string(),
        "Sten".to_string(),
        "Markus".to_string(),
        "Maris".to_string(),
        "Karl".to_string()
    ];
    pub static ref GENDERS: Vec<Gender> = vec![Gender::Male, Gender::Female];
}
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Item {
    Bread,
    Coin,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    items: HashMap<Item, i32>,
}
impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            items: HashMap::new(),
        }
    }
    pub fn get_coins(&self) -> i32 {
        let count = self.items.get(&Item::Coin).unwrap_or(&0);
        *count
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Stats {
    health: i8,
    hunger: u8,
    strength: u8,
    intelligence: u8,
    agility: u8,
}
#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum Faction {
    Empty,
    Hiisi,
    Virumaa,
    Pohjola,
    Tapiola,
    Kalevala,
    Novgorod,
}
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, PartialEq, Deserialize)]
pub enum Status {
    Talking,
    Fighting,
    Idle,
}
#[derive(Clone, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    WoodenWall,
}
#[derive(Clone, Serialize, Deserialize)]
pub enum EntityType {
    Human,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Coords {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Size {
    pub x: i32,
    pub y: i32,
}
impl Size {
    pub fn from(size: (i32, i32)) -> Size {
        Size {
            x: size.0,
            y: size.1,
        }
    }
}
impl Coords {
    pub fn from(coords: (f32, f32)) -> Coords {
        Coords {
            x: coords.0,
            y: coords.1,
        }
    }
    pub fn new() -> Coords {
        Coords { x: 0.0, y: 0.0 }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Entity {
    pub coords: Coords,
    pub ccoords: Coords,
    pub vel: (f32, f32),
    pub ang: f32,
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
            coords: Coords::new(),
            ccoords: Coords::new(),
            vel: (0.0, 0.0),
            ang: 0.0,
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
    pub fn gen_player(id: usize, x:f32, y:f32) -> Entity {
        Entity {
            coords: Coords::from((x,y)),
            ccoords: Coords::from(((x/ *CHUNK_SIZE as f32).floor(), (y / *CHUNK_SIZE as f32).floor())),
            vel: (0.0, 0.0),
            ang: 0.0,
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
        coords: Coords,
        vel: (f32, f32),
        etype: EntityType,
        stats: Stats,
        alignment: Alignment,
        name: String,
        gender: Gender,
	current_world: usize,
    ) -> Entity {
        Entity {
            coords: coords.clone(),
            ccoords: Coords::from(((coords.x / *CHUNK_SIZE as f32).floor(), (coords.y / *CHUNK_SIZE as f32).floor())),
            etype: etype,
            vel: (0.0, 0.0),
            ang: 0.0,
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

        if self.stats.hunger > 0 {
            self.stats.hunger -= 1;
        }
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if self.stats.hunger == 0 {
            if self.stats.health >= 0 {
                self.stats.health -= 2;
            } else {
                self.stats.health = 0;
            }
        }
        // resolve tasks
        //
        if self.tasks.build.1 {}
    }
    pub fn resolve_against(&mut self, other: &mut Entity, step_increment: i32) {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if dist(&self.coords, &other.coords) <= *VICINITY_DIST {
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
#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    pub coords: Coords,
    pub index: usize,
    pub size: Size,
    pub height: i32,
    pub ttype: TileType,
    pub holds: Option<Entity>,
    pub designed: Option<TileType>,
}

impl Tile {
    pub fn from(
        coords: Coords,
        index: usize,
        size: Size,
        height: i32,
        ttype: TileType,
        holds: Option<Entity>,
    ) -> Tile {
        Tile {
            coords,
            index,
            size,
            height,
            ttype,
            holds,
            designed: None,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    pub coords: Coords,
    pub index: usize,
}

impl Chunk {
    pub fn from(tiles: Vec<Tile>, entities: Vec<Entity>, coords: Coords, index: usize) -> Chunk {
        Chunk {
            tiles,
            entities,
            coords,
            index,
        }
    }
    pub fn new() -> Chunk {
        Chunk {
            tiles: vec![],
            entities: vec![],
            coords: Coords::new(),
            index: 0,
        }
    }
    pub fn resolve(&mut self, step_increment: i32) {
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
            faction = &Faction::Novgorod;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.1
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.2
        {
            faction = &Faction::Virumaa;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.2
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.3
        {
            faction = &Faction::Kalevala;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.3
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.4
        {
            faction = &Faction::Tapiola;
        } else if fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) > 0.4
            && fac_perlin.get([self.coords.x as f64 + 0.1, self.coords.y as f64 + 0.1]) < 0.5
        {
            faction = &Faction::Pohjola;
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
                    Coords::from((x as f32, y as f32)),
                    (0.0, 0.0),
                    EntityType::Human,
                    Stats::gen(),
                    Alignment::from(faction.clone()),
                    gen_human_name(faction.clone(), gender),
                    gender.clone(),
		    0,
                ))
            }
            tiles.push(Tile::from(
                Coords::from((x as f32, y as f32)),
                c as usize,
                Size::from((*TILE_SIZE as i32, *TILE_SIZE as i32)),
                height,
                TileType::Grass,
                None,
            ));
        }
        Chunk {
            tiles: tiles,
            entities: entities,
            coords: self.coords.clone(),
            index: self.index,
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
#[derive(Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
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
    pub fn update_chunk_with_entity(&mut self, entity: Entity) {
        let x_int = entity.ccoords.x as i32;
        let y_int = entity.ccoords.y as i32;
        let chunk = &mut self.chunks[(y_int * *WORLD_SIZE as i32 + x_int) as usize];
        
        // Try to find an entity with the same ID
        if let Some(existing_entity) = chunk.entities.iter_mut().find(|e| e.index == entity.index) {
            // Update the existing entity
	    println!("updateing {}", entity.coords.x);
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
            Coords::from((x, y)),
            c as usize,
        ));
    }
    chunks.par_iter_mut().for_each(|c| *c = c.gen(seed));
    let world = World::from(chunks);
    world
}
