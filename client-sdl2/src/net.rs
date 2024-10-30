use crate::worldgen::{Chunk, Entity};
use bincode::deserialize;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientData {
    pub entity: Entity,
}

pub async fn fetch_chunk(x: f32, y: f32, index: usize) -> Result<Chunk, Error> {
    let url = format!(
        "http://localhost:3000/chunk?x={}&y={}&index={}",
        x, y, index
    );
    let response = reqwest::get(&url).await?;

    let chunk = response.bytes().await?;
    let chunk_des = bincode::deserialize(&chunk).unwrap();
    Ok(chunk_des)
}

pub async fn send_client_data(client_data: ClientData) -> Result<(), Error> {
    let player_data = bincode::serialize(&client_data).unwrap();
    // Create reqwest client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    // Send POST request with binary data
    let response = client
        .post("http://localhost:3000/client_data")
        .header("Content-Type", "application/octet-stream")
        .body(player_data)
        .send()
        .await?;

    // Check if the request was successful
    if response.status().is_success() {
        //println!("Player data sent successfully!");
    } else {
        println!("Failed to send player data: {}", response.status());
    }

    Ok(())
}
