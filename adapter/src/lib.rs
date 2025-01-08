use crate::net::*;
use crate::util::{ClientData, ActionContent, ClientDataType};
use godot::classes::ISprite2D;
use godot::classes::Node;
use godot::classes::Sprite2D;
use godot::prelude::*;
use tokio::runtime::Runtime;
use crate::worldgen::{Entity, Chunk};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct GClientData {
    id: usize,
    coords: (f32,f32,f32),
}

struct DimensionerExtension;
#[gdextension]
unsafe impl ExtensionLibrary for DimensionerExtension {}
#[derive(GodotClass)]
#[class(base=Node)]
struct Net {}
#[godot_api]
impl INode for Net {
    fn init(base: Base<Node>) -> Self {
        Self {}
    }
}
#[godot_api]
impl Net {
    #[func]
    fn transfer(&mut self, data: String) -> String {
	let gdata: GClientData = serde_json::from_str(&data).expect("cdata deserialization failed");
	let cdata = ClientData::from(Entity::gen_player(gdata.id, gdata.coords.0, gdata.coords.1, gdata.coords.2), ActionContent::new(), ClientDataType::Chunk);
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let res = rt.block_on(async {
            send_client_data(cdata).await
        });
	serde_json::to_string(&res.unwrap()).expect("Could not parse chunks to string")
    }
}
pub mod lang;
pub mod math;
pub mod net;
pub mod util;
pub mod worldgen;
