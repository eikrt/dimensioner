use bincode;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::io;
use std::sync::Arc;
use std::thread;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task;
use tokio::time::{sleep, Duration};
use url::{form_urlencoded, Url};
use dimensioner_server::util::RenderMsg;
use dimensioner_server::worldgen::{worldgen, Entity, Camera, News, CHUNK_SIZE, WORLD_SIZE};
#[derive(Clone, Serialize, Deserialize, Debug)]
struct ClientData {
    entity: Entity
}
lazy_static! {
    pub static ref PARTITION_SIZE: usize = (*WORLD_SIZE as usize * *WORLD_SIZE as usize) / 16;
}

// Function to handle requests and route to the appropriate response
#[tokio::main]
async fn main() {
    // Create a broadcast channel (tx: sender, rx: receiver)
    let (tx, _rx) = broadcast::channel(256);
    let (tx_c, mut rx_c): (
        broadcast::Sender<ClientData>,
        broadcast::Receiver<ClientData>,
    ) = broadcast::channel(256);
    let mut worlds = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..1 {
        let seed = rng.gen_range(0..1000);
        worlds.push(worldgen(seed));
    }
    let mut state: Vec<RenderMsg> = vec![];
    let mut step = 0;
    let mut step_increment = 1;
    let mut vic_world = 0;
    let mut render = false;
    // Spawn a worker thread to send "world data" every few seconds
    task::spawn(async move {
        let mut counter = 0;
        loop {
            match rx_c.try_recv() {
                Ok(o) => {
		   worlds[0].update_chunk_with_entity(o.entity); 
		   
                }
                Err(e) => {}
            }
            worlds
                .par_iter_mut()
                .for_each(|c| c.resolve(step_increment));
            worlds
                .par_iter_mut()
                .for_each(|c| c.resolve_between(step_increment));
            let worlds_clone = worlds.clone();
            counter += 1;
            // Send the world data through the broadcast channel
            if let Err(_) = tx.send(worlds_clone) {
                println!("No active receivers left");
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
        }
    });

    // Create a Hyper service that will serve the data received from the broadcast channel
    let tx_c_clone = tx_c.clone();
    let make_svc = make_service_fn(move |_conn| {
        let mut rx = _rx.resubscribe();
	let val = tx_c_clone.clone();
        let service = service_fn(move |req: Request<Body>| {
            let mut rx = rx.resubscribe(); // Clone the receiver inside the async block
	    let tx_c_clone_clone = val.clone();
            async move {
                while rx.is_empty() {}
                match (req.method(), req.uri().path()) {
                    // Handle GET requests to /chunk
                    (&Method::GET, "/chunk") => {
                        let uri_string = req.uri().to_string();
                        let base_url = "http://dummy.com";
                        let request_url = Url::parse(base_url).unwrap().join(&uri_string).unwrap();
                        let params = request_url.query_pairs();

                        let mut x: Option<f32> = None;
                        let mut y: Option<f32> = None;
                        let mut index: Option<usize> = None;

                        for (key, value) in params {
                            match key.as_ref() {
                                "x" => x = value.parse().ok(),
                                "y" => y = value.parse().ok(),
                                "index" => index = value.parse().ok(),
                                _ => {}
                            }
                        }
                        match rx.recv().await {
                            Ok(worlds_clone) => {
                                if let (Some(ix), Some(x_val), Some(y_val)) = (index, x, y) {
                                    let chunk = worlds_clone[ix].fetch_chunk_x_y(x_val, y_val);
                                    Ok::<_, Infallible>(Response::new(Body::from(
                                        bincode::serialize(&chunk).unwrap(),
                                    )))
                                } else {
                                    Ok::<_, Infallible>(
                                        Response::builder()
                                            .status(StatusCode::BAD_REQUEST)
                                            .body(Body::from("Missing parameters"))
                                            .unwrap(),
                                    )
                                }
                            }
                            Err(_) => Ok::<_, Infallible>(
                                Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(Body::from("Failed to receive world data"))
                                    .unwrap(),
                            ),
                        }
                    }

                    // Handle GET requests to /worlds
                    (&Method::GET, "/worlds") => match rx.recv().await {
                        Ok(worlds_clone) => Ok::<_, Infallible>(Response::new(Body::from(
                            bincode::serialize(&*worlds_clone).unwrap(),
                        ))),
                        Err(_) => Ok::<_, Infallible>(
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to receive world data"))
                                .unwrap(),
                        ),
                    },

                    // Handle POST requests to /client_data
                    (&Method::POST, "/client_data") => {
                        let whole_body = hyper::body::to_bytes(req.into_body()).await;
                        match whole_body {
                            Ok(bytes) => {
                                // Deserialize using bincode
                                match bincode::deserialize::<ClientData>(&bytes) {
                                    Ok(client_data) => {
                                        let _ = tx_c_clone_clone.send(client_data);
                                        Ok::<_, Infallible>(Response::new(Body::from(
                                            "Client data received successfully",
                                        )))
                                    }
                                    Err(_) => Ok::<_, Infallible>(
                                        Response::builder()
                                            .status(StatusCode::BAD_REQUEST)
                                            .body(Body::from("Invalid data format"))
                                            .unwrap(),
                                    ),
                                }
                            }
                            Err(_) => Ok::<_, Infallible>(
                                Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(Body::from("Failed to read request body"))
                                    .unwrap(),
                            ),
                        }
                    }

                    // Default 404 response for other routes
                    _ => Ok::<_, Infallible>(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("Not Found"))
                            .unwrap(),
                    ),
                }
            }
        });

        async move { Ok::<_, Infallible>(service) }
    });

    // Bind the server to an address
    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    // Print server info
    println!("Listening on http://{}", addr);

    // Run the server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
