use std::io::{Read, Write};
use std::str;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Receiver, Sender};

use anyhow::Result;

type Conn = Arc<TcpStream>;

enum Event {
    Connected(Conn),
    Message(Box<[u8]>),
}

async fn client(conn: Conn, events: Sender<Event>) -> Result<()> {
    let _ = events
        .send(Event::Connected(conn.clone()))
        .await
        .map_err(|err| {
            println!("[ERROR]: failed to connect client to server, {:?}", err);
        });

    let mut buf = [0; 64];
    loop {
        match conn.as_ref().read(&mut buf).await {
            Ok(0) => {
                println!("[INFO]: client disconnected");
                break;
            }
            Ok(n) => {
                let bytes = buf[0..n].iter().cloned().filter(|b| *b >= 32).collect();
                let _ = events.send(Event::Message(bytes)).await.map(|err| {
                    eprintln!(
                        "[ERROR]: Failed to send bytes to server from client, {:?}",
                        err
                    );
                });
            }
            Err(err) => {
                println!(
                    "[ERROR]: An error occured while reading bytes from client, {:?}",
                    err
                );
            }
        }
    }

    return Ok(());
}

async fn server(mut events: Receiver<Event>) -> Result<()> {
    while let Some(event) = events.recv().await {
        match event {
            Event::Connected(conn) => {
                let _ = writeln!(conn.as_ref(), "Welcome to the chatroom").map_err(|err| {
                    eprintln!("[ERROR]: failed to write welcome message, {:?}", err);
                });
                println!("[INFO]: client connected.");
            }
            Event::Message(message) => {
                if let Ok(message) = str::from_utf8(&message) {
                    println!("[INFO]: client sent message: {message}");
                }
            }
        }
    }
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    let (tx, rx) = mpsc::channel::<Event>(10);

    tokio::spawn(server(rx));

    println!("[INFO]: server started at 127.0.0.1:3000");

    loop {
        match listener.accept().await {
            Ok(conn) => {
                let conn = Arc::new(conn);
                let sender = tx.clone();
                let _ = client(conn, sender).await;
            }
            Err(_) => {
                eprintln!("[ERROR]: failed to accept new connections");
                break;
            }
        }
    }

    return Ok(());
}
