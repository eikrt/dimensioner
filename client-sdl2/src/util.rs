use crate::worldgen::{Camera, Entity, Chunk, News};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug)]
pub struct RenderMsg {
    pub chunk: Chunk,
    pub news: News,
}
impl RenderMsg {
    pub fn from(chunk: Chunk, news: News) -> RenderMsg {
        RenderMsg {
            chunk: chunk,
            news: news,
        }
    }
}
#[derive(Clone, Debug)]
pub struct MainMsg {
    pub camera: Camera,
    pub player: Option<Entity>,
    pub ok: bool,
}
impl MainMsg {
    pub fn from(camera: Camera, entity: Option<Entity>, ok: bool) -> MainMsg {
        MainMsg {
            camera: camera,
	    player: entity,
            ok: ok,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClientMsg {
    pub player: Entity,
    pub action: ActionType,
}
impl ClientMsg{
    pub fn from(player: Entity, action: ActionType) -> ClientMsg{
        ClientMsg {
            player: player,
	    action: action,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientData {
    pub entity: Entity,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ActionType {
    Empty,
    ConstructCannon,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ActionData {
    pub action: ActionType,
    pub entity: Entity,
}
