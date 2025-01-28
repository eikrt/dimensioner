use crate::math::{dist_f32_f32, dist_f32_i32};
use crate::util::ActionType;
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use noise::{NoiseFn, Perlin};
use rand::prelude::SliceRandom;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
lazy_static! {
    pub static ref WORLD_SIZE: u32 = 64;
    pub static ref CHUNK_SIZE: u32 = 32;
    pub static ref TILE_SIZE: u32 = 16;
    pub static ref NOISE_SCALE: f64 = 64.0;
    pub static ref VICINITY_DIST: i32 = 2;
    pub static ref HUMAN_NAMES_F: Vec<String> = vec![
        "Kate".to_string(),
        "Elsa".to_string(),
        "Karen".to_string(),
        "Jade".to_string(),
        "Alice".to_string(),
        "Sophia".to_string(),
        "Emma".to_string(),
        "Olivia".to_string(),
        "Ava".to_string(),
        "Isabella".to_string(),
        "Mia".to_string(),
        "Amelia".to_string(),
        "Charlotte".to_string(),
        "Harper".to_string(),
        "Abigail".to_string(),
        "Emily".to_string(),
        "Ella".to_string(),
        "Scarlett".to_string(),
        "Grace".to_string(),
        "Chloe".to_string(),
        "Victoria".to_string(),
        "Zoe".to_string(),
        "Lily".to_string(),
        "Hannah".to_string(),
        "Sofia".to_string(),
        "Madison".to_string(),
        "Aria".to_string(),
        "Layla".to_string(),
        "Eleanor".to_string(),
        "Ellie".to_string(),
        "Stella".to_string(),
        "Hazel".to_string(),
        "Violet".to_string(),
        "Aurora".to_string(),
        "Savannah".to_string(),
        "Bella".to_string(),
        "Luna".to_string(),
        "Nora".to_string(),
        "Penelope".to_string(),
        "Riley".to_string(),
        "Mila".to_string(),
        "Elizabeth".to_string(),
        "Brooklyn".to_string(),
        "Aubrey".to_string(),
        "Anna".to_string(),
        "Addison".to_string(),
        "Willow".to_string(),
        "Lucy".to_string(),
        "Audrey".to_string(),
        "Evelyn".to_string(),
        "Camila".to_string(),
        "Claire".to_string(),
        "Samantha".to_string(),
        "Elena".to_string(),
        "Ariana".to_string(),
        "Autumn".to_string(),
        "Quinn".to_string(),
        "Ivy".to_string(),
        "Naomi".to_string(),
        "Sarah".to_string(),
        "Jasmine".to_string(),
        "Delilah".to_string(),
        "Maya".to_string(),
        "Katherine".to_string(),
        "Julia".to_string(),
        "Madelyn".to_string(),
        "Sydney".to_string(),
        "Faith".to_string(),
        "Lillian".to_string(),
        "Holly".to_string(),
        "Vivian".to_string(),
        "Iris".to_string(),
        "Alexa".to_string(),
        "Clara".to_string(),
        "Paisley".to_string(),
        "Everly".to_string(),
        "Fiona".to_string(),
        "Rowan".to_string(),
        "Tessa".to_string(),
        "Eden".to_string(),
        "Sienna".to_string(),
        "Valerie".to_string(),
        "Leah".to_string(),
        "Kayla".to_string(),
        "Melanie".to_string(),
        "Brooke".to_string(),
        "Isla".to_string(),
        "Mckenzie".to_string(),
        "Lila".to_string(),
        "Reagan".to_string(),
        "Emery".to_string(),
        "Daisy".to_string(),
        "Juliet".to_string(),
        "Alina".to_string(),
        "Genevieve".to_string(),
        "Cora".to_string(),
        "Adeline".to_string(),
        "Rosalie".to_string(),
        "Piper".to_string(),
        "Margot".to_string()
    ];
    pub static ref HUMAN_NAMES_M: Vec<String> = vec![
        "John".to_string(),
        "Jack".to_string(),
        "Jacques".to_string(),
        "Tom".to_string(),
        "Arnold".to_string(),
        "James".to_string(),
        "Michael".to_string(),
        "William".to_string(),
        "David".to_string(),
        "Joseph".to_string(),
        "Charles".to_string(),
        "Thomas".to_string(),
        "Daniel".to_string(),
        "Matthew".to_string(),
        "Henry".to_string(),
        "Christopher".to_string(),
        "Andrew".to_string(),
        "Joshua".to_string(),
        "Nathan".to_string(),
        "Alexander".to_string(),
        "Ryan".to_string(),
        "Ethan".to_string(),
        "Samuel".to_string(),
        "Elijah".to_string(),
        "Benjamin".to_string(),
        "Logan".to_string(),
        "Noah".to_string(),
        "Lucas".to_string(),
        "Liam".to_string(),
        "Oliver".to_string(),
        "Jacob".to_string(),
        "Caleb".to_string(),
        "Gabriel".to_string(),
        "Nicholas".to_string(),
        "Zachary".to_string(),
        "Aaron".to_string(),
        "Tyler".to_string(),
        "Evan".to_string(),
        "Nathaniel".to_string(),
        "Connor".to_string(),
        "Mason".to_string(),
        "Aiden".to_string(),
        "Isaac".to_string(),
        "Dylan".to_string(),
        "Hunter".to_string(),
        "Austin".to_string(),
        "Adrian".to_string(),
        "Dominic".to_string(),
        "Jordan".to_string(),
        "Parker".to_string(),
        "Blake".to_string(),
        "Sebastian".to_string(),
        "Carter".to_string(),
        "Justin".to_string(),
        "Brandon".to_string(),
        "Cole".to_string(),
        "Xavier".to_string(),
        "Miles".to_string(),
        "Gavin".to_string(),
        "Elliott".to_string(),
        "Oscar".to_string(),
        "Finn".to_string(),
        "Tristan".to_string(),
        "Julian".to_string(),
        "Leo".to_string(),
        "Harrison".to_string(),
        "Vincent".to_string(),
        "Maxwell".to_string(),
        "Grant".to_string(),
        "Hudson".to_string(),
        "Asher".to_string(),
        "Silas".to_string(),
        "Wyatt".to_string(),
        "Kai".to_string(),
        "Lincoln".to_string(),
        "Ryder".to_string(),
        "Weston".to_string(),
        "Bryce".to_string(),
        "Emmett".to_string(),
        "Reid".to_string(),
        "Brody".to_string(),
        "Zane".to_string(),
        "Ezekiel".to_string(),
        "Theo".to_string(),
        "Sawyer".to_string(),
        "Levi".to_string(),
        "Jasper".to_string(),
        "Colton".to_string(),
        "Orion".to_string(),
        "Quinn".to_string(),
        "Dean".to_string(),
        "Axel".to_string(),
        "Malcolm".to_string(),
        "Rowan".to_string(),
        "Beckett".to_string(),
        "Gideon".to_string(),
        "Ronan".to_string(),
        "Pierce".to_string(),
        "Sterling".to_string(),
        "Tobias".to_string()
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
} // Implement AddAssign trait for FlooredF32
impl AddAssign for HashableF32 {
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

impl Mul for HashableF32 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        HashableF32(self.0 * other.0)
    }
}
// Implement other necessary traits for convenience
impl PartialEq for HashableF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.floor() == other.0.floor()
    }
}
impl TryInto<i32> for HashableF32 {
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
        HashableF32(self.as_f32().sqrt())
    }
    pub fn as_i32(&self) -> i32 {
        self.0.floor() as i32
    }
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}
#[derive(Clone, Debug)]
pub struct Camera {
    pub coords: Coords_f32,
    pub ccoords: Coords_i32,
    pub render_distance_w: i32,
    pub render_distance_h: i32,
    pub scale_x: f32,
    pub scale_y: f32,
}
impl Camera {
    pub fn new() -> Camera {
        Camera {
            coords: Coords_f32::new(),
            ccoords: Coords_i32::new(),
            render_distance_w: 128 as i32,
            render_distance_h: 128 as i32,
            scale_x: 1.0,
            scale_y: 1.0,
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
    fire: (u8, bool),
    explode: (u8, bool),
    migrate: (u8, bool),
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
            fire: (0, false),
            explode: (0, false),
            migrate: (0, false),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct DialogueNode {
    pub content: String,
    pub requirement_stats: Option<Stats>,
    pub requirement_item: Option<Item>,
}
impl DialogueNode {
    pub fn from(
        content: String,
        requirement_stats: Option<Stats>,
        requirement_item: Option<Item>,
    ) -> DialogueNode {
        DialogueNode {
            content: content,
            requirement_stats: requirement_stats,
            requirement_item: requirement_item,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct DialogueTree {
    pub message: DialogueNode,
    pub answer: DialogueNode,
    pub nodes: Vec<Option<DialogueTree>>,
    pub giver: Option<Box<Entity>>,
}

impl DialogueTree {
    pub fn from(
        message: DialogueNode,
        answer: DialogueNode,
        nodes: Vec<Option<DialogueTree>>,
        giver: Option<Box<Entity>>,
    ) -> DialogueTree {
        DialogueTree {
            message: message,
            answer: answer,
            nodes: nodes,
            giver: giver,
        }
    }
    pub fn moo(giver: Option<Box<Entity>>) -> DialogueTree {
        DialogueTree::from(
            DialogueNode::from(format!("Moo!",), None, None),
            DialogueNode::from(format!("Moo moo!",), None, None),
            vec![Some(DialogueTree::from(
                DialogueNode::from("...".to_string(), None, None),
                DialogueNode::from("Moo!".to_string(), None, None),
                vec![],
                giver.clone(),
            ))],
            giver.clone(),
        )
    }
    pub fn investigate_plant(giver: Option<Box<Entity>>) -> DialogueTree {
        DialogueTree::from(
            DialogueNode::from(format!("You see a plant...",), None, None),
            DialogueNode::from(format!("You see a plant...",), None, None),
            vec![Some(DialogueTree::from(
                DialogueNode::from(
                    "Investigate plant".to_string(),
                    Some(Stats::gen_plant()),
                    None,
                ),
                DialogueNode::from(
                    format!(
                        "You get the following remarks: {:?}",
                        giver.clone().unwrap().body_sheet()
                    ),
                    Some(Stats::gen_plant()),
                    None,
                ),
                vec![],
                giver.clone(),
            ))],
            giver.clone(),
        )
    }
    pub fn investigate_crop(giver: Option<Box<Entity>>) -> DialogueTree {
        DialogueTree::from(
            DialogueNode::from(format!("You see a plant...",), None, None),
            DialogueNode::from(format!("You see a plant...",), None, None),
            vec![Some(DialogueTree::from(
                DialogueNode::from(
                    "Investigate plant".to_string(),
                    Some(Stats::gen_crop()),
                    None,
                ),
                DialogueNode::from(
                    format!(
                        "You get the following remarks: {:?}",
                        giver.clone().unwrap().get_sheet()
                    ),
                    Some(Stats::gen_crop()),
                    None,
                ),
                vec![],
                giver.clone(),
            ))],
            giver.clone(),
        )
    }
    pub fn plague(giver: Option<Box<Entity>>) -> DialogueTree {
        DialogueTree::from(
            DialogueNode::from(
                format!(
                    "Hello, my name is {}... What can I do for you?",
                    giver.clone().unwrap().name
                ),
                None,
                None,
            ),
            DialogueNode::from(
                format!(
                    "Hello, my name is {}... What can I do for you?",
                    giver.clone().unwrap().name
                ),
                None,
                None,
            ),
            vec![
                Some(DialogueTree::from(
                    DialogueNode::from("I'm looking for jobs.".to_string(), None, None),
                    DialogueNode::from("I have problems with my crops... Something keeps plaguing them... Maybe you could do something about it?".to_string(), None, None),
                    vec![Some(DialogueTree::from(
                        DialogueNode::from("Can you tell me what's wrong with them?".to_string(), None, None),
                        DialogueNode::from("I ain't a botanist, so I can't help you with that...".to_string(), None, None),
                        vec![],
                        giver.clone(),
                    ))],
                    giver.clone(),
                )),
                Some(DialogueTree::from(
                    DialogueNode::from("What's your story?".to_string(), None, None),
                    DialogueNode::from("I used to be a soldier... I'd rather not talk about my past...".to_string(), None, None),
                    vec![],
                    giver.clone(),
                )),
                Some(DialogueTree::from(
                    DialogueNode::from("What's going on around here?".to_string(), None, None),
                    DialogueNode::from("Not much I afraid...".to_string(), None, None),
                    vec![],
                    giver.clone(),
                )),
            ],
            giver.clone(),
        )
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Hash)]
pub struct Quest {}
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug, Hash)]
pub enum Class {
    Detective,
    Mailcarrier,
    Businessman,
    Chemist,
    Engineer,
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
    pub items: Vec<(Item, u32)>,
}
impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            items: vec![(Item::Coin, 1)],
        }
    }
    pub fn get_coins(&self) -> u32 {
        return self.items[0].1;
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Stats {
    pub health: i32,
    pub hunger: u8,
    pub strength: u8,
    pub intelligence: u8,
    pub charisma: u8,
    pub agility: u8,
    pub senses: u8,
    pub endurance: u8,
    pub luck: u8,
    pub botanist: u8,
    pub zoology: u8,
    pub ecology: u8,
    pub explosives: u8,
    pub mechanic: u8,
    pub social: u8,
    pub doctor: u8,
    pub sneak: u8,
    pub marksmanship: u8,
    pub cook: u8,
    pub fisher: u8,
    pub sailor: u8,
    pub unarmed: u8,
    pub mining: u8,
    pub mathematic: u8,
    pub gambler: u8,
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
            charisma: 10,
            senses: 10,
            endurance: 10,
            luck: 10,
            botanist: 10,
            zoology: 10,
            ecology: 10,
            explosives: 10,
            mechanic: 10,
            social: 10,
            doctor: 10,
            sneak: 10,
            marksmanship: 10,
            cook: 10,
            fisher: 10,
            sailor: 10,
            unarmed: 10,
            mining: 10,
            mathematic: 10,
            gambler: 10,
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
            charisma: rng.gen_range(0..10),
            senses: rng.gen_range(0..10),
            endurance: rng.gen_range(0..10),
            luck: rng.gen_range(0..10),
            botanist: rng.gen_range(0..10),
            zoology: rng.gen_range(0..10),
            ecology: rng.gen_range(0..10),
            explosives: rng.gen_range(0..10),
            mechanic: rng.gen_range(0..10),
            social: rng.gen_range(0..10),
            doctor: rng.gen_range(0..10),
            sneak: rng.gen_range(0..10),
            marksmanship: rng.gen_range(0..10),
            cook: rng.gen_range(0..10),
            fisher: rng.gen_range(0..10),
            sailor: rng.gen_range(0..10),
            unarmed: rng.gen_range(0..10),
            mining: rng.gen_range(0..10),
            mathematic: rng.gen_range(0..10),
            gambler: rng.gen_range(0..10),
        }
    }
    pub fn gen_from_class(class: &Class) -> Stats {
        let mut rng = rand::thread_rng();
        match class {
            Class::Detective => Stats {
                health: 100,
                hunger: 100,
                strength: 5,
                intelligence: 8,
                agility: 6,
                charisma: 4,
                senses: 6,
                endurance: 2,
                luck: 5,
                botanist: 10,
                zoology: 5,
                ecology: 5,
                explosives: 10,
                mechanic: 5,
                social: 15,
                doctor: 10,
                sneak: 20,
                marksmanship: 15,
                cook: 5,
                fisher: 5,
                sailor: 5,
                unarmed: 10,
                mining: 5,
                mathematic: 10,
                gambler: 20,
            },
            Class::Mailcarrier => Stats {
                health: 100,
                hunger: 100,
                strength: 5,
                intelligence: 5,
                agility: 5,
                charisma: 5,
                senses: 5,
                endurance: 5,
                luck: 5,
                botanist: 5,
                zoology: 5,
                ecology: 5,
                explosives: 10,
                mechanic: 10,
                social: 15,
                doctor: 5,
                sneak: 5,
                marksmanship: 5,
                cook: 15,
                fisher: 10,
                sailor: 10,
                unarmed: 10,
                mining: 10,
                mathematic: 10,
                gambler: 5,
            },
            Class::Chemist => Stats {
                health: 100,
                hunger: 100,
                strength: 2,
                intelligence: 8,
                agility: 5,
                charisma: 5,
                senses: 8,
                endurance: 3,
                luck: 5,
                botanist: 25,
                zoology: 25,
                ecology: 25,
                explosives: 20,
                mechanic: 5,
                social: 5,
                doctor: 15,
                sneak: 5,
                marksmanship: 5,
                cook: 15,
                fisher: 10,
                sailor: 5,
                unarmed: 5,
                mining: 15,
                mathematic: 10,
                gambler: 5,
            },
            Class::Businessman => Stats {
                health: 100,
                hunger: 100,
                strength: 3,
                intelligence: 7,
                agility: 2,
                charisma: 9,
                senses: 5,
                endurance: 2,
                luck: 8,
                botanist: 5,
                zoology: 5,
                ecology: 5,
                explosives: 15,
                mechanic: 5,
                social: 30,
                doctor: 5,
                sneak: 5,
                marksmanship: 10,
                cook: 10,
                fisher: 15,
                sailor: 20,
                unarmed: 10,
                mining: 5,
                mathematic: 10,
                gambler: 30,
            },
            Class::Engineer => Stats {
                health: 100,
                hunger: 100,
                strength: 5,
                intelligence: 5,
                agility: 5,
                charisma: 3,
                senses: 5,
                endurance: 6,
                luck: 5,
                botanist: 10,
                zoology: 10,
                ecology: 10,
                explosives: 20,
                mechanic: 30,
                social: 5,
                doctor: 10,
                sneak: 5,
                marksmanship: 15,
                cook: 15,
                fisher: 10,
                sailor: 10,
                unarmed: 5,
                mining: 15,
                mathematic: 15,
                gambler: 10,
            },
        }
    }
    pub fn gen_plant() -> Stats {
        let mut rng = rand::thread_rng();
        Stats {
            health: 100,
            hunger: 100,
            strength: 5,
            intelligence: 5,
            agility: 5,
            charisma: 3,
            senses: 5,
            endurance: 6,
            luck: 5,
            botanist: 5,
            zoology: rng.gen_range(0..10),
            ecology: rng.gen_range(0..10),
            explosives: rng.gen_range(0..10),
            mechanic: rng.gen_range(0..10),
            social: rng.gen_range(0..10),
            doctor: rng.gen_range(0..10),
            sneak: rng.gen_range(0..10),
            marksmanship: rng.gen_range(0..10),
            cook: rng.gen_range(0..10),
            fisher: rng.gen_range(0..10),
            sailor: rng.gen_range(0..10),
            unarmed: rng.gen_range(0..10),
            mining: rng.gen_range(0..10),
            mathematic: rng.gen_range(0..10),
            gambler: rng.gen_range(0..10),
        }
    }
    pub fn gen_crop() -> Stats {
        let mut rng = rand::thread_rng();
        Stats {
            health: 100,
            hunger: 100,
            strength: 5,
            intelligence: 5,
            agility: 5,
            charisma: 3,
            senses: 5,
            endurance: 6,
            luck: 5,
            botanist: 15,
            zoology: rng.gen_range(0..10),
            ecology: rng.gen_range(0..10),
            explosives: rng.gen_range(0..10),
            mechanic: rng.gen_range(0..10),
            social: rng.gen_range(0..10),
            doctor: rng.gen_range(0..10),
            sneak: rng.gen_range(0..10),
            marksmanship: rng.gen_range(0..10),
            cook: rng.gen_range(0..10),
            fisher: rng.gen_range(0..10),
            sailor: rng.gen_range(0..10),
            unarmed: rng.gen_range(0..10),
            mining: rng.gen_range(0..10),
            mathematic: rng.gen_range(0..10),
            gambler: rng.gen_range(0..10),
        }
    }
    pub fn stat_sheet_hard(&self) -> String {
        format!(
            "Strength: {}\nIntelligence: {}\nAgility: {}\nCharisma: {}\nSenses: {}\nEndurance: {}\nLuck: {}",
            self.strength,
            self.intelligence,
            self.agility,
            self.charisma,
            self.senses,
            self.endurance,
            self.luck
        )
        .to_string()
    }
    pub fn stat_sheet_soft(&self) -> String {
        format!(
            "Botanist: {}\nZoology: {}\nEcology: {}\nExplosives: {}\nMechanic: {}\nSocial: {}\nDoctor: {}\nSneak: {}\nMarksmanship: {}\nCook: {}\nFisher: {}\nSailor: {}\nUnarmed: {}\nMining: {}\nMathematic: {}\nGambler: {}",
            self.botanist,
            self.zoology,
            self.ecology,
            self.explosives,
            self.mechanic,
            self.social,
            self.doctor,
	    self.sneak,
	    self.marksmanship,
	    self.cook,
	    self.fisher,
	    self.sailor,
	    self.unarmed,
	    self.mining,
	    self.mathematic,
	    self.gambler,
        )
        .to_string()
    }
}
#[derive(Clone, Serialize, PartialEq, Deserialize, Debug, Hash)]
pub enum Status {
    Talking,
    Fighting,
    Idle,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq)]
pub enum TileType {
    Grass,
    Water,
    Sand,
    StoneSand,
    FarmLand,
    WetLand,
    Asphalt,
    Salt,
    Wood,
    Concrete,
    Granite,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq)]
pub enum EntityType {
    Cactus,
    Tumbleweed,
    Human,
    Cow,
    Cannon,
    Cauliflower,
    Lily,
    Tulip,
    Stone,
    Shell,
    Road,
    Explosion,
    Landmine,
    Car,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq)]
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
#[derive(Clone, Serialize, Deserialize, Debug, Hash, PartialEq)]
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
        Coords_f32 {
            x: HashableF32(0.0),
            y: HashableF32(0.0),
            z: HashableF32(0.0),
        }
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
pub enum DiseaseType {
    Healthy,
    FusariumWilt,
    VerticilliumWilt,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub enum BodyPartType {
    Head,
    LeftArm,
    RightArm,
    Torso,
    LeftLeg,
    RightLeg,
    Stem,
    Areoles,
    Flower,
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct BodyPart {
    bptype: BodyPartType,
    dttype: DiseaseType,
    health: i32,
}
impl BodyPart {
    pub fn from(bptype: BodyPartType, dttype: DiseaseType, health: i32) -> BodyPart {
        BodyPart {
            bptype: bptype,
            dttype: dttype,
            health: health,
        }
    }
    pub fn get_sheet(&self) -> Vec<String>{
	vec![format!("Type: {:?}", self.bptype), format!("Disease: {:?}", self.dttype), format!("Health: {}", self.health)]
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub struct Entity {
    pub coords: Coords_f32,
    pub ccoords: Coords_i32,
    pub current_action: ActionType,
    pub vel: Coords_f32,
    pub ang: HashableF32,
    pub traj: HashableF32,
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
    pub linked_entity_id: u64,
    pub class: Class,
    pub experience: i32,
    pub level: i32,
    pub dialogue: Option<DialogueTree>,
    pub parts: Vec<BodyPart>,
}
impl Entity {
    pub fn new(index: usize) -> Entity {
        Entity {
            coords: Coords_f32::new(),
            ccoords: Coords_i32::new(),
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            current_action: ActionType::Empty,
            etype: EntityType::Human,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: index,
            name: "".to_string(),
            class: Class::Mailcarrier,
            gender: Gender::Female,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
        }
    }
    pub fn body_sheet(&self) -> Vec<String> {
	let mut vec: Vec<String> = vec![];
	for s in &self.parts {
	    vec.extend(s.get_sheet());
	}
	vec
    }
    pub fn gen_player(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            // coords: Coords_f32::from((0.0,0.0,0.0)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Human,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "Player".to_string(),
            gender: Gender::Female,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
        }
    }
    pub fn gen_npc(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Human,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: gen_human_name(Faction::Marine, &Gender::Male),
            class: Class::Mailcarrier,
            gender: Gender::Male,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            level: 1,
            experience: 0,
            parts: vec![],
            dialogue: None,
        }
    }
    pub fn gen_shell(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Shell,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "Shell".to_string(),
            class: Class::Mailcarrier,
            gender: Gender::Other,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
        }
    }
    pub fn gen_explosion(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Explosion,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "Explosion".to_string(),
            gender: Gender::Other,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
        }
    }
    pub fn gen_car(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Car,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "Car".to_string(),
            gender: Gender::Other,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
        }
    }
    pub fn gen_crop(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Cauliflower,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "".to_string(),
            gender: Gender::Other,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![BodyPart::from(
                BodyPartType::Stem,
                DiseaseType::Healthy,
                100,
            )],
        }
    }
    pub fn gen_sick_plant(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Cauliflower,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "".to_string(),
            gender: Gender::Other,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![BodyPart::from(
                BodyPartType::Stem,
                DiseaseType::VerticilliumWilt,
                10,
            )],
        }
    }
    pub fn gen_cattle(id: usize, x: f32, y: f32, z: f32) -> Entity {
        Entity {
            coords: Coords_f32::from((x, y, z)),
            ccoords: Coords_i32::from((
                (HashableF32(x) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(y) / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (HashableF32(z) / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            etype: EntityType::Cow,
            stats: Stats::new(),
            status: Status::Idle,
            alignment: Alignment::new(),
            inventory: Inventory::new(),
            index: id,
            name: "".to_string(),
            gender: Gender::Other,
            tasks: Tasks::new(),
            current_world: 0,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
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
            ccoords: Coords_i32::from((
                (coords.x / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (coords.y / HashableF32(*TILE_SIZE as f32) / HashableF32(*CHUNK_SIZE as f32))
                    .as_i32(),
                (coords.z / HashableF32(*CHUNK_SIZE as f32)).as_i32(),
            )),
            current_action: ActionType::Empty,
            etype: etype,
            vel: Coords_f32::new(),
            ang: HashableF32(0.0),
            traj: HashableF32(0.0),
            stats: stats,
            status: Status::Idle,
            index: index,
            alignment: alignment,
            inventory: Inventory::new(),
            name: name,
            gender: gender,
            tasks: Tasks::new(),
            current_world: current_world,
            linked_entity_id: 0,
            class: Class::Mailcarrier,
            level: 1,
            experience: 0,
            dialogue: None,
            parts: vec![],
        }
    }
    pub fn get_sheet(&mut self) -> Vec<String> {
	vec![format!("{}", self.name), format!("{:?}", self.etype)]
    }
    pub fn fire(&mut self) {
        self.tasks.fire = (1, true);
    }
    pub fn resolve(&mut self, step_increment: i32) {
        // movement

        self.ccoords.x =
            (self.coords.x / HashableF32(*CHUNK_SIZE as f32) / HashableF32(*TILE_SIZE as f32))
                .as_i32();
        self.ccoords.y =
            (self.coords.y / HashableF32(*CHUNK_SIZE as f32) / HashableF32(*TILE_SIZE as f32))
                .as_i32();
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
        if self.etype == EntityType::Shell {
            self.coords.x += self.vel.x;
            self.coords.y += self.vel.y;
            self.coords.z += self.vel.z;
            self.vel.z -= HashableF32(0.0064);
        }
        if self.etype == EntityType::Explosion {
            self.stats.health -= 16;
        }
        if self.etype == EntityType::Cannon {
            let now = SystemTime::now();
            if now
                .duration_since(UNIX_EPOCH)
                .expect("Failed to assert time")
                .as_millis()
                % 256
                == 0
            {
                self.fire();
            }
        }
        if self.tasks.build.1 {}
    }
    pub fn resolve_against(&mut self, other: &mut Entity, step_increment: i32) {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10);
        if dist_f32_f32(&self.coords, &other.coords) <= *VICINITY_DIST {
            if other.etype == EntityType::Explosion {
                self.stats.health -= 50;
            }
            if self.etype == EntityType::Landmine {
                self.stats.health = -1;
            }
            if other.status == Status::Fighting {
                let dmg = other.stats.strength * roll;
                self.stats.health -= dmg as i32;
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
    pub fn get_sheet(&mut self) -> Vec<String> {
        return vec![format!("{:?}", self.ttype).to_string()];
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
    pub timezone: u8,
    pub observed: bool,
}

impl Chunk {
    pub fn from(
        tiles: Vec<Tile>,
        entities: Vec<Entity>,
        coords: Coords_i32,
        index: usize,
        hash: u64,
        timezone: u8,
    ) -> Chunk {
        Chunk {
            tiles,
            entities,
            coords,
            index,
            hash,
            timezone: timezone,
            observed: false,
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
            timezone: 0,
            observed: false,
        }
    }
    pub fn resolve(&mut self, step_increment: i32) -> Vec<Entity> {
        if !self.observed {
            return vec![];
        }
        let mut hasher = Sha256::new();
        hasher.update(bincode::serialize(&self.tiles).unwrap());
        hasher.update(bincode::serialize(&self.entities).unwrap());
        let mut added_entities: Vec<Entity> = vec![];
        let mut leftover_entities: Vec<Entity> = vec![];
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
                if entity.tasks.fire.1 {
                    let mut s = Entity::gen_shell(
                        entity.index + 1,
                        entity.coords.x.as_f32(),
                        entity.coords.y.as_f32(),
                        entity.coords.z.as_f32(),
                    );
                    s.traj = entity.traj;
                    s.vel.x = HashableF32(entity.ang.0.sin() * 1.0) * HashableF32(1.0);
                    s.vel.y = HashableF32(-entity.ang.0.cos() * 1.0) * HashableF32(1.0);
                    s.vel.z = HashableF32(entity.traj.0.cos() * 1.0) * HashableF32(0.5);
                    added_entities.push(s);
                    entity.tasks.fire.1 = false;
                }

                if entity.ccoords != self.coords {
                    leftover_entities.push(entity.clone());
                    entity.tasks.migrate.1 = true;
                }
            }
            for e in &mut self.entities {
                if e.etype == EntityType::Explosion {
                    continue;
                }
                if e.etype == EntityType::Landmine {
                    if e.stats.health <= 0 {
                        let mut s = Entity::gen_explosion(
                            e.index + 1,
                            e.coords.x.as_f32(),
                            e.coords.y.as_f32(),
                            e.coords.z.as_f32(),
                        );
                        added_entities.push(s);
                    }
                }
                if e.etype == EntityType::Shell {
                    for t in &mut self.tiles {
                        if HashableF32(t.coords.x as f32).as_i32()
                            == (e.coords.x / HashableF32(*TILE_SIZE as f32)).as_i32()
                            && HashableF32(t.coords.y as f32).as_i32()
                                == (e.coords.y / HashableF32(*TILE_SIZE as f32)).as_i32()
                        {
                            if e.coords.z.as_i32() < t.coords.z {
                                e.stats.health = -1;
                                t.coords.z -= 1;
                                let mut s = Entity::gen_explosion(
                                    e.index + 1,
                                    e.coords.x.as_f32(),
                                    e.coords.y.as_f32(),
                                    e.coords.z.as_f32(),
                                );
                                added_entities.push(s);
                            }
                        }
                    }
                }
            }
            self.entities.extend(added_entities.clone());
            self.entities = self
                .entities
                .iter()
                .filter(|e| e.stats.health > 0 && !e.tasks.migrate.1)
                .cloned()
                .collect();
        }
        leftover_entities
    }
    pub fn gen(&mut self, seed: u32, img: Option<&DynamicImage>) -> Chunk {
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
            let a_x = x * *TILE_SIZE as i32;
            let a_y = y * *TILE_SIZE as i32;
            let a = 2.0;
            let n1 = perlin.get([
                (x as f64) / *NOISE_SCALE + 0.1,
                (y as f64) / *NOISE_SCALE + 0.1,
            ]) * a;

            let n2 = perlin2.get([
                (x as f64) / *NOISE_SCALE * 2.0 + 0.1,
                (y as f64) / *NOISE_SCALE * 2.0 + 0.1,
            ]) * a;

            let n3 = perlin3.get([
                (x as f64) / (*NOISE_SCALE * 8.0) + 0.1,
                (y as f64) / (*NOISE_SCALE * 8.0) + 0.1,
            ]) * a;
            let height = match img {
                Some(s) => {
                    let color = s.get_pixel(x as u32, y as u32).0[1];
                    let mut height: f32;
                    if color == 0 {
                        height = -1.0;
                    } else {
                        height = 1.0;
                    }
                    height
                }
                None => (n1 + n2 + n3 + rng.gen_range(-0.5..1.0)) as f32,
            };
            let mut ttype = TileType::Sand;
            let mut biome = "desert";
            if height < 0.0 {
                ttype = TileType::Water;
            }
            let gender = GENDERS.choose(&mut rand::thread_rng()).unwrap();
            if biome == "heartland" {
                if height >= 0.0 && rng.gen_range(0..32) == 1 {
                    entities.push(Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Human,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        gen_human_name(faction.clone(), gender),
                        gender.clone(),
                        0,
                    ))
                }
                if height >= 0.0 && rng.gen_range(0..64) == 1 {
                    entities.push(Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Cauliflower,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        "Cauliflower".to_string(),
                        gender.clone(),
                        0,
                    ))
                }
                if height >= 0.0 && rng.gen_range(0..64) == 1 {
                    entities.push(Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Lily,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        "Lily".to_string(),
                        gender.clone(),
                        0,
                    ))
                }
                if height >= 0.0 && rng.gen_range(0..64) == 1 {
                    entities.push(Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Stone,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        "Stone".to_string(),
                        gender.clone(),
                        0,
                    ))
                }
                if height >= 0.0 && rng.gen_range(0..64) == 1 {
                    entities.push(Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Tulip,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        "Tulip".to_string(),
                        gender.clone(),
                        0,
                    ))
                }
            } else if biome == "desert" {
                if height >= 0.0 && rng.gen_range(0..64) == 1 {
                    let mut plant = Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Cactus,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        "".to_string(),
                        gender.clone(),
                        0,
                    );

                    plant.parts = vec![
                        BodyPart::from(BodyPartType::Stem, DiseaseType::Healthy, 100),
                        BodyPart::from(BodyPartType::Areoles, DiseaseType::Healthy, 100),
                    ];
                    plant.dialogue = Some(DialogueTree::investigate_plant(Some(Box::new(
                        plant.clone(),
                    ))));
                    entities.push(plant)
                }
                if height >= 0.0 && rng.gen_range(0..64) == 1 {
                    let mut plant = Entity::from(
                        c as usize,
                        Coords_f32::from((a_x as f32, a_y as f32, height as f32)),
                        (0.0, 0.0, 0.0),
                        EntityType::Tumbleweed,
                        Stats::gen(),
                        Alignment::from(faction.clone()),
                        "".to_string(),
                        gender.clone(),
                        0,
                    );

                    plant.parts = vec![
                        BodyPart::from(BodyPartType::Stem, DiseaseType::Healthy, 100),
                        BodyPart::from(BodyPartType::Flower, DiseaseType::Healthy, 100),
                    ];
                    plant.dialogue = Some(DialogueTree::investigate_plant(Some(Box::new(
                        plant.clone(),
                    ))));
                    entities.push(plant)
                }
            }
            tiles.push(Tile::from(
                Coords_i32::from((x, y, height as i32)),
                c as usize,
                Size::from((*TILE_SIZE as i32, *TILE_SIZE as i32, *TILE_SIZE as i32)),
                ttype,
                None,
            ));
        }

        for c in 0..(*CHUNK_SIZE as i32 * *CHUNK_SIZE as i32) {
            let x_c = c % (*CHUNK_SIZE as i32) + self.coords.x as i32 * *CHUNK_SIZE as i32;
            let y_c = (c / *CHUNK_SIZE as i32) + self.coords.y as i32 * *CHUNK_SIZE as i32;
            let a_x = x_c * *TILE_SIZE as i32;
            let a_y = y_c * *TILE_SIZE as i32;
            // Randomly decide whether to create a shack at this location
            let mut discard = false;
            if rng.gen_range(0..1000) == 1 {
                // Ensure the shack fits within the bounds of the chunk
                let start_x = *CHUNK_SIZE as i32 / 8;
                let start_y = *CHUNK_SIZE as i32 / 8;
                // Iterate through the 8x8 area to create the shack

                for y in start_y..(start_y + 8) {
                    for x in start_x..(start_x + 8) {
                        let tile_index = (y * *CHUNK_SIZE as i32 + x) as usize;
                        if let Some(t) = tiles.get_mut(tile_index) {
                            if t.ttype == TileType::Concrete
                                || t.ttype == TileType::Water
                                || t.ttype == TileType::FarmLand
                            {
                                discard = true;
                            }
                        }
                    }
                }
                if discard {
                    continue;
                }
                for y in start_y..(start_y + 4) {
                    for x in start_x..(start_x + 4) {
                        let tile_index = (y * *CHUNK_SIZE as i32 + x) as usize;
                        if let Some(t) = tiles.get_mut(tile_index) {
                            if rng.gen_range(0..8) == 1 {
                                let mut npc = Entity::gen_npc(
                                    rng.gen_range(0..100000),
                                    t.coords.x as f32 * *TILE_SIZE as f32,
                                    t.coords.y as f32 * *TILE_SIZE as f32,
                                    ((1 * *TILE_SIZE as i32) as f32),
                                );
                                npc.dialogue =
                                    Some(DialogueTree::plague(Some(Box::new(npc.clone()))));
                                entities.push(npc);
                            }
                            t.ttype = TileType::Concrete;
                        }
                    }
                }
                for y in (start_y)..(start_y + 4) {
                    for x in (start_x + 5)..(start_x + 4 + 5) {
                        let tile_index = (y * *CHUNK_SIZE as i32 + x) as usize;
                        if let Some(t) = tiles.get_mut(tile_index) {
                            if rng.gen_range(0..4) == 1 {
                                let mut plant = Entity::gen_sick_plant(
                                    rng.gen_range(0..100000),
                                    t.coords.x as f32 * *TILE_SIZE as f32,
                                    t.coords.y as f32 * *TILE_SIZE as f32,
                                    ((1 * *TILE_SIZE as i32) as f32),
                                );
                                plant.dialogue = Some(DialogueTree::investigate_plant(Some(
                                    Box::new(plant.clone()),
                                )));
                                entities.push(plant);
                            }
                            t.ttype = TileType::FarmLand;
                        }
                    }
                }
                for y in (start_y + 5)..(start_y + 4 + 5) {
                    for x in (start_x + 5)..(start_x + 4 + 5) {
                        let tile_index = (y * *CHUNK_SIZE as i32 + x) as usize;
                        if let Some(t) = tiles.get_mut(tile_index) {
                            if rng.gen_range(0..8) == 1 {
                                let mut cattle = Entity::gen_cattle(
                                    rng.gen_range(0..100000),
                                    t.coords.x as f32 * *TILE_SIZE as f32,
                                    t.coords.y as f32 * *TILE_SIZE as f32,
                                    ((1 * *TILE_SIZE as i32) as f32),
                                );

                                cattle.dialogue =
                                    Some(DialogueTree::moo(Some(Box::new(cattle.clone()))));
                                entities.push(cattle);
                            }
                            t.ttype = TileType::Grass;
                        }
                    }
                }
            }
        }
        Chunk {
            tiles: tiles,
            entities: entities,
            coords: self.coords.clone(),
            index: self.index,
            hash: 0,
            timezone: self.timezone,
            observed: false,
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
            .for_each(|e| coin_count += e.inventory.get_coins());

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
    pub time: u64,
}
impl World {
    pub fn from(chunks: Vec<Chunk>, time: u64) -> World {
        World { chunks, time }
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
    pub fn fetch_chunk_x_y_mut(&mut self, x: f32, y: f32) -> &mut Chunk {
        let x_int = x as i32;
        let y_int = y as i32;
        &mut self.chunks[(y_int * *WORLD_SIZE as i32 + x_int) as usize]
    }
    pub fn update_chunk_with_entity(&mut self, mut entity: Entity) {
        let x_int = entity.ccoords.x as i32;
        let y_int = entity.ccoords.y as i32;
        if x_int < 0
            || y_int < 0
            || x_int > *WORLD_SIZE as i32 - 1
            || y_int > *WORLD_SIZE as i32 - 1
        {
            return;
        }
        entity.ccoords.x = (entity.coords.x
            / HashableF32(*TILE_SIZE as f32)
            / HashableF32(*CHUNK_SIZE as f32)
            / HashableF32(1 as f32))
        .as_i32();
        entity.ccoords.y = (entity.coords.y
            / HashableF32(*TILE_SIZE as f32)
            / HashableF32(*CHUNK_SIZE as f32)
            / HashableF32(1 as f32))
        .as_i32();
        let chunk = &mut self.chunks[(y_int * *WORLD_SIZE as i32 + x_int) as usize];
        chunk.observed = true;
        // Try to find an entity with the same ID
        if let Some(existing_entity) = chunk.entities.iter_mut().find(|e| e.index == entity.index) {
            // Update the existing entity
            *existing_entity = entity;
        } else {
            // Add the new entity
            chunk.entities.push(entity);
        }
    }
    pub fn resolve(&mut self, delta: f32, step_increment: i32) {
        self.time += delta as u64;
        let mut leftover_entities = vec![];
        self.chunks.iter_mut().for_each(|c| {
            let los = c.resolve(step_increment);
            leftover_entities.extend(los);
            c.observed = false;
        });
        for le in &leftover_entities {
            self.update_chunk_with_entity(le.clone());
        }
    }
    pub fn resolve_between(&mut self, step_increment: i32) {}
}
pub fn worldgen(seed: u32) -> World {
    let image_path = "data/map/globe.gif";
    let img = ImageReader::open(image_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");
    let mut chunks: Vec<Chunk> = vec![];
    for c in 0..((*WORLD_SIZE * *WORLD_SIZE) as i32) {
        let x = (c % *WORLD_SIZE as i32) as f32;
        let y = (c / *WORLD_SIZE as i32) as f32;
        let timezone: u8 = ((*WORLD_SIZE as f32 - x) / 10.0) as u8;
        chunks.push(Chunk::from(
            vec![],
            vec![],
            Coords_i32::from((x as i32, y as i32, 0)),
            c as usize,
            0,
            timezone,
        ));
    }
    chunks.par_iter_mut().for_each(|c| *c = c.gen(seed, None));
    let world = World::from(chunks, 0);
    world
}
pub fn globegen() -> World {
    let image_path = "data/map/globe.gif";
    let mut img = ImageReader::open(image_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");
    img = img.resize_exact(
        *WORLD_SIZE * *CHUNK_SIZE,
        *WORLD_SIZE * *CHUNK_SIZE,
        FilterType::Nearest,
    );
    let (width, height) = img.dimensions();
    let mut chunks: Vec<Chunk> = vec![];
    for c in 0..((*WORLD_SIZE * *WORLD_SIZE) as i32) {
        let x = (c % *WORLD_SIZE as i32) as f32;
        let y = (c / *WORLD_SIZE as i32) as f32;
        let timezone: u8 = ((*WORLD_SIZE as f32 - x) / 10.0) as u8;
        chunks.push(Chunk::from(
            vec![],
            vec![],
            Coords_i32::from((x as i32, y as i32, 0)),
            c as usize,
            0,
            timezone,
        ));
    }
    chunks
        .par_iter_mut()
        .for_each(|c| *c = c.gen(0, Some(&img)));
    let world = World::from(chunks, 0);
    world
}
