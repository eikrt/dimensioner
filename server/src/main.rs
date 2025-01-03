use bincode;
use dimensioner_server::util::RenderMsg;
use dimensioner_server::util::{ActionData, ActionType, ClientData, ClientDataType};
use dimensioner_server::worldgen::*;
use lazy_static::lazy_static;
use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tokio::time::{sleep, Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::io::{self, Write, Read};
use std::convert::TryInto;

lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel
    let (tx, _rx) = unbounded();
    let (tx_c, mut rx_c): (
        Sender<ClientData>,
        Receiver<ClientData>,
    ) = unbounded();
    let (tx_c_a, mut rx_c_a): (
        Sender<ClientData>,
        Receiver<ClientData>,
    ) = unbounded();

    let mut worlds = Arc::new(Mutex::new(vec![]));
    let mut rng = rand::thread_rng();
    for _ in 0..1 {
        let seed = rng.gen_range(0..1000);
        //worlds.push(worldgen(seed));
	worlds.lock().unwrap().push(globegen());
    }

    let mut step_increment = 1;

    // Spawn a worker thread to send "world data" every few seconds
    task::spawn(async move {
        let mut rng = StdRng::from_entropy(); // Separate RNG instance for this task
        loop {
            if let Ok(o) = rx_c_a.try_recv() {
                match o.entity.current_action {
                    ActionType::Empty => {},
		    ActionType::Refresh => {},
                    ActionType::ConstructCannon => {
			let mut coords = Coords_f32::new();
			coords.x = HashableF32((o.entity.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			coords.y = HashableF32((o.entity.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
                        let mut entity = Entity::from(
                            rng.gen_range(0..1000) as usize,
                            coords,
                            (0.0, 0.0, 0.0),
                            EntityType::Cannon,
                            Stats::gen(),
                            Alignment::from(Faction::Marine),
                            gen_human_name(Faction::Marine, &Gender::Other),
                            Gender::Other,
                            0,
                        );
			entity.ang = o.action.ang;
                        worlds.lock().unwrap()[0].update_chunk_with_entity(entity);
                    },
                    ActionType::ConstructRoad => {
			let mut coords = Coords_f32::new();
			coords.x = HashableF32((o.entity.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			coords.y = HashableF32((o.entity.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
                        let mut entity = Entity::from(
                            rng.gen_range(0..1000) as usize,
                            coords,
                            (0.0, 0.0, 0.0),
                            EntityType::Road,
                            Stats::gen(),
                            Alignment::from(Faction::Marine),
                            gen_human_name(Faction::Marine, &Gender::Other),
                            Gender::Other,
                            0,
                        );
			entity.ang = o.action.ang;
                        worlds.lock().unwrap()[0].update_chunk_with_entity(entity);
                    },
                    ActionType::ConstructLandmine => {
			let mut coords = Coords_f32::new();
			coords.x = HashableF32((o.entity.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			coords.y = HashableF32((o.entity.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
                        let mut entity = Entity::from(
                            rng.gen_range(0..1000) as usize,
                            coords,
                            (0.0, 0.0, 0.0),
                            EntityType::Landmine,
                            Stats::gen(),
                            Alignment::from(Faction::Marine),
                            gen_human_name(Faction::Marine, &Gender::Other),
                            Gender::Other,
                            0,
                        );
			entity.ang = o.action.ang;
                        worlds.lock().unwrap()[0].update_chunk_with_entity(entity);
                    },
                    ActionType::ConstructShell => {
			let mut coords = Coords_f32::new();
			coords.x = HashableF32((o.entity.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			coords.y = HashableF32((o.entity.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			let mut entity = Entity::gen_shell(rng.gen_range(0..1000), coords.x.as_f32(),coords.y.as_f32(), coords.z.as_f32());
			entity.traj = entity.traj;
			entity.vel.x = HashableF32(o.action.ang.as_f32().sin() * 1.0) * HashableF32(1.0);
			entity.vel.y = HashableF32(-o.action.ang.as_f32().cos() * 1.0) * HashableF32(1.0);
			entity.vel.z = HashableF32(o.action.traj.as_f32().cos() * 1.0) * HashableF32(0.5);
			entity.ang = o.action.ang;
                        worlds.lock().unwrap()[0].update_chunk_with_entity(entity);
                    },
                    ActionType::ConstructCar => {
			let mut coords = Coords_f32::new();
			coords.x = HashableF32((o.entity.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			coords.y = HashableF32((o.entity.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			let mut entity = Entity::gen_car(rng.gen_range(0..1000), coords.x.as_f32(),coords.y.as_f32(), coords.z.as_f32());
			entity.traj = entity.traj;
			entity.vel.x = HashableF32(o.action.ang.as_f32().sin() * 1.0) * HashableF32(1.0);
			entity.vel.y = HashableF32(-o.action.ang.as_f32().cos() * 1.0) * HashableF32(1.0);
			entity.vel.z = HashableF32(o.action.traj.as_f32().cos() * 1.0) * HashableF32(0.5);
			entity.ang = o.action.ang;
                        worlds.lock().unwrap()[0].update_chunk_with_entity(entity);
                    }
                    ActionType::Interact => {
			let mut coords = Coords_f32::new();
			coords.x = HashableF32((o.entity.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			coords.y = HashableF32((o.entity.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			for mut e in &mut worlds.lock().unwrap()[0].fetch_chunk_x_y_mut(o.entity.ccoords.x as f32, o.entity.ccoords.y as f32).entities {
			    let mut coords_e = Coords_f32::new();
			    coords_e.x = HashableF32((e.coords.x.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			    coords_e.y = HashableF32((e.coords.y.as_f32() / *TILE_SIZE as f32).floor() * *TILE_SIZE as f32);
			    if coords.x == coords_e.x && coords.y == coords_e.y {
				e.linked_entity_id = o.entity.index as u64;
			    }
			}
                    }
                }
            }

            worlds.lock().unwrap().par_iter_mut().for_each(|c| c.resolve(10.0, step_increment));
            worlds.lock().unwrap().par_iter_mut().for_each(|c| c.resolve_between(step_increment));

            if let Ok(o) = rx_c.try_recv() {
		for mut e in &mut worlds.lock().unwrap()[0].fetch_chunk_x_y_mut(o.entity.ccoords.x as f32, o.entity.ccoords.y as f32).entities {
		    if e.linked_entity_id as u64 == o.entity.index as u64 {
			e.coords = o.entity.coords.clone();
		    }
		}
                worlds.lock().unwrap()[0].update_chunk_with_entity(o.entity);
            }
            //let worlds_clone = worlds.clone();
            if let Err(_) = tx.send(Arc::clone(&worlds)) {
                println!("No active receivers left");
            }

            sleep(Duration::from_millis(1000 / 120)).await;
        }
    });

    // Start a TCP server
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on 127.0.0.1:3000");

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let tx_c_clone = tx_c.clone();
            let tx_c_a_clone = tx_c_a.clone();
            let mut rx = _rx.clone();

            task::spawn(handle_connection(stream, tx_c_clone, tx_c_a_clone, rx));
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    tx_c: Sender<ClientData>,
    tx_c_a: Sender<ClientData>,
    mut rx: Receiver<Arc<Mutex<Vec<World>>>>,
) {
    let mut buffer = [0; 65536];

    loop {
        let read_result = stream.read(&mut buffer).await;
	let mut result_client_data: Option<ClientData> = None;
        match read_result {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                // Deserialize the received data
                let incoming_data: Result<ClientData, _> = bincode::deserialize(&buffer[..n]);
                if let Ok(client_data) = incoming_data {
		    result_client_data = Some(client_data.clone());
                    let _ = tx_c.send(client_data.clone());
                    let _ = tx_c_a.send(client_data);
                } else if let Ok(action_data) = bincode::deserialize(&buffer[..n]) {
                    let _ = tx_c_a.send(action_data);
                }
		else {
		    let client_data_error = bincode::deserialize::<ClientData>(&buffer[..n]).err();
		    let action_data_error = bincode::deserialize::<ActionData>(&buffer[..n]).err();
		    eprintln!(
			"Failed to parse received data:\n - Raw bytes: {:?}\n - ClientData error: {:?}\n - ActionData error: {:?}",
			&buffer[..n],
			client_data_error,
			action_data_error,
		    );
		}
		
                // Send back a response with the current world state
                if let Ok(worlds) = rx.recv() {
		    let c = result_client_data.unwrap();
		    if c.entity.ccoords.x as f32 >= 0.0 && c.entity.ccoords.y as f32 >= 0.0 && (c.entity.ccoords.x as f32 ) < (*WORLD_SIZE as f32) && (c.entity.ccoords.y as f32 ) < (*WORLD_SIZE as f32) {
			match c.data_type {
			    ClientDataType::Chunk => {
				let serialized_worlds = bincode::serialize(&worlds.lock().unwrap()[0].fetch_chunk_x_y(c.entity.ccoords.x as f32, c.entity.ccoords.y as f32 )).unwrap();
				let _ = stream.write_all(&serialized_worlds).await;
			    },
			    ClientDataType::Refresh => {
				let serialized_worlds = bincode::serialize(&worlds.lock().unwrap()[0].fetch_chunk_x_y(c.entity.ccoords.x as f32, c.entity.ccoords.y as f32 )).unwrap();
				let _ = stream.write_all(&serialized_worlds).await;
			    }
			}
		    }
		    else {

			let serialized_worlds = bincode::serialize(&worlds.lock().unwrap()[0].fetch_chunk_x_y(0.0, 0.0)).unwrap();
			let _ = stream.write_all(&serialized_worlds).await;
		    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}
