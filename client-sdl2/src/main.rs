use async_std::task;
use crossbeam::channel::unbounded;
use dimensioner_client_sdl2::net::{send_client_data};
use dimensioner_client_sdl2::plot::plot;
use dimensioner_client_sdl2::renderer::render_server;
use dimensioner_client_sdl2::util::{
    ActionData, ActionType, ActionContent, ClientData, ClientMsg, MainMsg, RenderMsg,
};
use dimensioner_client_sdl2::worldgen::{worldgen, Camera, Entity, News, CHUNK_SIZE, WORLD_SIZE};

use rand::Rng;
use rayon::prelude::*;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
    pub static ref VIEW_DISTANCE: usize = 4;
}
fn main() {
    let (tx, rx) = unbounded();
    let (tx2, rx2): (
        crossbeam::channel::Sender<MainMsg>,
        crossbeam::channel::Receiver<MainMsg>,
    ) = unbounded();
    let (tx3, rx3) = unbounded();
    let (tx4, rx4): (
        crossbeam::channel::Sender<ClientMsg>,
        crossbeam::channel::Receiver<ClientMsg>,
    ) = unbounded();
    let (tx5, rx5): (
        crossbeam::channel::Sender<ClientMsg>,
        crossbeam::channel::Receiver<ClientMsg>,
    ) = unbounded();
    let (tx6, rx6): (
        crossbeam::channel::Sender<ClientMsg>,
        crossbeam::channel::Receiver<ClientMsg>,
    ) = unbounded();
    let (tx_c, rx_c): (
        crossbeam::channel::Sender<ClientMsg>,
        crossbeam::channel::Receiver<ClientMsg>,
    ) = unbounded();
    let rx4_clone = rx4.clone();
    let rx2_clone = rx2.clone();
    let rx3_clone = rx3.clone();
    let tx3_clone = tx3.clone();
    let mut state: Arc<Mutex<Vec<RenderMsg>>> = Arc::new(Mutex::new(vec![]));
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..=100_000);
    let mut player: Arc<Mutex<Entity>> = Arc::new(Mutex::new(Entity::gen_player(random_number, 0.0, 0.0, 0.0)));
    let player_id = player.lock().unwrap().index;
    let mut step = 0;
    let mut step_increment = 1;
    let mut camera = Camera::new();
    let mut vic_world = 0;
    let mut render = false;
    let mut current_action_content = ActionContent::new();
    let mut p1 = player.clone();
    let mut p2 = player.clone();
    let mut tx3_c = tx3.clone();
    let mut tx_c_c = tx_c.clone();
    thread::spawn(move || loop {
        let _ = tx3.send(ClientMsg::from(player.lock().unwrap().clone(), ActionContent::new()));
        let _ = tx_c.send(ClientMsg::from(player.lock().unwrap().clone(), current_action_content.clone()));
        if let Ok(p) = rx4.recv() {
	    let mut player_from = p.player;
	    player_from.current_action = p.action.action_type.clone();
	    current_action_content = p.action.clone();
	    *player.lock().unwrap() = player_from;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    });
    let state_clone_clone = state.clone();
    let tx5_c = tx5.clone();
    let mut x_i = 0;
    let mut y_i = 0;
    thread::spawn(move || loop {
        // Continuously read from the channel until there are no more messages
        let mut latest_message = None;
        let mut latest_message_server = None;
        while let Ok(p) = rx_c.try_recv() {
            latest_message = Some(p);
            while let Ok(p) = rx6.try_recv() {
		latest_message_server = Some(p);
            }
        }
        // Process only the latest message, if any
        if let Some(p) = latest_message {
	    let mut e = p.player.clone();
	    let e_i = e.index;
	    match latest_message_server {
		Some(s) => {e.ccoords = s.player.ccoords;},
		None => {}
	    };
            let s = ClientData { entity: e, x_i: x_i, y_i: y_i};
            let a = ActionData {
                entity: p.player.clone(),
                action: p.action.action_type.clone(),
            };
	    x_i += 1;
	    if x_i > *VIEW_DISTANCE as i32 {
		y_i += 1;
		x_i = 0;
	    }
	    if y_i > *VIEW_DISTANCE as i32 {
		x_i = 0;
		y_i = 0;
	    }
            let result = task::block_on(send_client_data(s));
	    match result {
                Ok(chunk) => {
		    if let Some(chunk) = chunk {
			chunk.entities.clone().into_iter().find(|e| {
			    if e.index == player_id
			    {
				tx6.send(ClientMsg::from(e.clone(), ActionContent::new()));
			    }
			    false
			}
			);
                        state_clone_clone.lock().unwrap().push(RenderMsg::from(chunk.clone(), chunk.inquire_news()));
		    }
                },
                Err(e) => eprintln!("Error fetching chunk: {}", e)
            };
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    );
    thread::spawn(move || {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(e) => e,
            Err(e) => panic!("End..."),
        };
        match input.as_str() {
            "" => "joinks",
            &_ => todo!(),
        }
    });
    let mut partition = 0;
    let mut state_clone = Arc::clone(&state);
    let mut p: Option<Entity> = None;
    let mut p_server_from: Option<Entity> = None;
    let mut p_server_local: Option<Entity> = None;
    let mut state_clone = Arc::clone(&state);
    thread::spawn(move || loop {
        let _ = tx.send(state.lock().unwrap().clone());
        step += step_increment;
        if let Some(ref p) = p {
            if let Some(ref p_s) = p_server_local {
                for i in (p_s.ccoords.x as i32 - (*(VIEW_DISTANCE) as i32))
                    ..((p_s.ccoords.x as i32 + *VIEW_DISTANCE as i32) as i32)
                {
                    for j in (p_s.ccoords.y as i32 - (*(VIEW_DISTANCE) as i32))
                        ..((p_s.ccoords.y as i32 + *VIEW_DISTANCE as i32) as i32)
                    {
                        if i < 0 || j < 0 || i > *WORLD_SIZE as i32 || j > *WORLD_SIZE as i32 {
                            continue;
                        }
                        // let result = task::block_on(fetch_chunk(j as f32, i as f32, 0));
                        // match result {
                        //     Ok(chunk) => {
                        //         let mut state = state.lock().unwrap();
                        //         if let Some(index) = state
                        //             .iter()
                        //             .position(|c| c.chunk.coords.x == j && c.chunk.coords.y == i)
                        //         {
                        //             let chunk_clone = chunk.clone();
                        //             for e in &chunk_clone.entities {
                        //                 if e.index == p.index {
                        //                     p_server_from = Some(e.clone());
                        //                 }
                        //             }
                        //             if state[index].chunk.hash != chunk.hash {
                        //                 state.remove(index);
                        //                 state.push(RenderMsg::from(
                        //                     chunk.clone(),
                        //                     chunk.inquire_news(),
                        //                 ));
                        //                 //println!("Replaced chunk at ({}, {}) with new data.", j, i);
                        //             }
                        //         } else {
                        //             state
                        //                 .push(RenderMsg::from(chunk.clone(), chunk.inquire_news()));
                        //             println!("Added new chunk at ({}, {})", j, i);
                        //         }
                        //     }
                        //     Err(e) => eprintln!("Error fetching chunk: {}", e),
                        // }
			
                    }
                }
            }
        } else {
            /*let result = task::block_on(fetch_chunk(0.0, 0.0, 0));
            match result {
                Ok(chunk) => {
                    state_clone
                        .lock()
                        .unwrap()
                        .push(RenderMsg::from(chunk.clone(), chunk.inquire_news()));
                }
                Err(e) => eprintln!("Error fetching chunk: {}", e),
            }*/
        }
        p_server_local = p_server_from.clone();
        if let Ok(x) = rx2_clone.recv() {
            camera = x.camera;
            p = x.player;
            p_server_local = match p_server_local {
                Some(s) => Some(s),
                None => p.clone(),
            };
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        partition += 1;
    });
    render_server(&tx2, &rx, &tx4, &rx3);
}
