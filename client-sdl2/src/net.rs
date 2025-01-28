use crate::worldgen::{Chunk, Entity};
use crate::util::{ActionData, ClientData};
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use bincode;
use tokio::net::TcpStream;
use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::task;
use std::sync::Arc;


pub async fn send_client_data(client_data: ClientData) -> Result<Option<Vec<Chunk>>, io::Error> {
    // Serialize the ClientData to binary format
    let serialized_data = bincode::serialize(&client_data).expect("Failed to serialize ClientData");
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:3000").await?;

    // Split the TcpStream into reader and writer
    let (mut reader, mut writer) = tokio::io::split(stream);

    // Spawn a task for writing the client data
    let write_task = task::spawn(async move {
        writer.write_all(&serialized_data).await?;
        writer.flush().await?;
        Ok::<Option<Vec<Chunk>>, io::Error>(None)
    });

    // Spawn a task for reading the response
    let read_task = task::spawn(async move {
        let mut buffer = vec![0; 65536]; // Allocate a buffer for the incoming response
	let mut chunk = None;
        match reader.read(&mut buffer).await {
            Ok(0) => {
                eprintln!("Server closed the connection.");
            }
            Ok(n) => {
                let response: Result<Vec<Chunk>, _> = bincode::deserialize(&buffer[..n]);
                match response {
                    Ok(data) => {
                        //println!("Received response: {:?}", data);
			chunk = Some(data);
                    }
                    Err(e) => {
                   //     eprintln!("Failed to parse server response with error {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
            }
        }
        Ok::<Option<Vec<Chunk>>, io::Error>(chunk)
    });

    // Await both tasks
    let _ = write_task.await?;
    let chunk = read_task.await?;
    match chunk {
	Ok(Some(c)) => Ok(Some(c)),
	Ok(None) => Ok(None),
	Err(e) => {eprintln!("{}", e); Ok(None)}
    }
}
