use crate::worldgen::{Camera, Entity, Chunk, News};

#[derive(Clone)]
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
#[derive(Clone)]
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

#[derive(Clone)]
pub struct ClientMsg {
    pub player: Entity,
}
impl ClientMsg{
    pub fn from(player: Entity) -> ClientMsg{
        ClientMsg {
            player: player,
        }
    }
}
