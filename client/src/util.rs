use crate::worldgen::{Camera, Entity, Chunk, Coords_i32, News, HashableF32};
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
    pub action: ActionContent,
}
impl ClientMsg{
    pub fn from(player: Entity, action: ActionContent) -> ClientMsg{
        ClientMsg {
            player: player,
	    action: action,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClientDataType {
    Chunk,
    Refresh,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientData {
    pub entity: Entity,
    pub action: ActionContent,
    pub data_type: ClientDataType,
    pub ccoords: Coords_i32,
}
impl ClientData {
    pub fn new() -> ClientData {
	ClientData {
	    entity: Entity::gen_player(0,0.0,0.0,0.0),
	    action: ActionContent::new(),
	    data_type: ClientDataType::Chunk, 
	    ccoords: Coords_i32::from((0,0,0)),
	}
    }
    pub fn from(entity: Entity, action: ActionContent, data_type: ClientDataType, ccoords: Coords_i32) -> ClientData {
	ClientData {
	    ccoords: ccoords,
	    entity: entity,
	    action: action, 
	    data_type: data_type,
	}
    }
}
#[derive(Hash, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ActionType {
    Empty,
    Refresh,
    ConstructCannon,
    ConstructRoad,
    ConstructShell,
    ConstructLandmine,
    ConstructCar,
    Interact,
}

#[derive(Hash, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ActionContent {
    pub action_type: ActionType,
    pub ang: HashableF32,
    pub traj: HashableF32,
}
impl ActionContent {
    pub fn new() -> ActionContent {
	ActionContent {
	    action_type: ActionType::Empty,
	    ang: HashableF32(0.0),
	    traj: HashableF32(0.0)
	}
    }
    pub fn from(action_type: ActionType, ang: HashableF32, traj: HashableF32) -> ActionContent {
	ActionContent {
	    action_type: action_type,
	    ang: ang,
	    traj: traj,
	}
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ActionData {
    pub action: ActionType,
    pub entity: Entity,
}
