use async_std::task;
use crossbeam::channel::unbounded;
use dimensioner_client_sdl2::net::send_client_data;
use dimensioner_client_sdl2::plot::plot;
use dimensioner_client_sdl2::renderer_opengl::render_server;
use dimensioner_client_sdl2::util::{
    ActionContent, ActionData, ActionType, ClientData, ClientDataType, ClientMsg, MainMsg,
    RenderMsg,
};
use dimensioner_client_sdl2::worldgen::{
    worldgen, globegen, Camera, Entity, News, CHUNK_SIZE, TILE_SIZE, WORLD_SIZE,
};

use rand::Rng;
use rayon::prelude::*;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
    pub static ref VIEW_DISTANCE: usize = 0;
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
    let mut player: Arc<Mutex<Entity>> = Arc::new(Mutex::new(Entity::gen_player(
        random_number,
        (*TILE_SIZE * *CHUNK_SIZE * *WORLD_SIZE / 2) as f32,
        (*TILE_SIZE * *CHUNK_SIZE * *WORLD_SIZE / 2) as f32,
        0.0,
    )));

    let player_clone = Arc::clone(&player);
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
    let mut player_state = Arc::new(Mutex::new(player.clone()));
    thread::spawn(move || loop {
        let _ = tx3.send(ClientMsg::from(
            player.lock().unwrap().clone(),
            ActionContent::new(),
        ));
        let _ = tx_c.send(ClientMsg::from(
            player.lock().unwrap().clone(),
            current_action_content.clone(),
        ));
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
    let s_clone = state.lock().unwrap().clone();
    let tx4_clone = tx4.clone();
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
                Some(s) => {
                    player_clone.lock().unwrap().ccoords = s.player.ccoords.clone();
                    player_clone.lock().unwrap().stats = s.player.stats.clone();
                    e.ccoords = s.player.ccoords;
                    e.stats = s.player.stats;
                }
                None => {}
            };
            let s = ClientData {
                entity: e,
                action: p.action.clone(),
                data_type: ClientDataType::Chunk,
            };
            let a = ActionData {
                entity: p.player.clone(),
                action: p.action.action_type.clone(),
            };
            let result = task::block_on(send_client_data(s));
            match result {
                Ok(chunk) => {
                    if let Some(chunk) = chunk {
                        chunk.entities.clone().into_iter().find(|e| {
                            if e.index == player_id {
                                tx6.send(ClientMsg::from(e.clone(), ActionContent::new()));
                            }
                            false
                        });

                        for (k, v) in s_clone.clone().into_iter().enumerate() {
                            if s_clone.clone().clone()[k].chunk.hash != chunk.hash {
                                state_clone_clone.lock().unwrap().remove(k);
                                state_clone_clone
                                    .lock()
                                    .unwrap()
                                    .push(RenderMsg::from(chunk.clone(), chunk.inquire_news()));
                                println!("Replaced chunk at ({}, {}) with new data.", 0, 0);
                            }
                        }
                        state_clone_clone.lock().unwrap().clear();
                        state_clone_clone
                            .lock()
                            .unwrap()
                            .push(RenderMsg::from(chunk.clone(), chunk.inquire_news()));
                    }
                }
                Err(e) => eprintln!("Error fetching chunk: {}", e),
            };
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    });
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
        state.lock().unwrap().clear();
        step += step_increment;
        if let Some(ref p) = p {
            if let Some(ref p_s) = p_server_local {}
        } else {
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
