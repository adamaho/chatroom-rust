use std::io::Write;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt;

use anyhow::Result;

async fn client(mut conn: TcpStream) -> Result<()> {
    let mut buf = Vec::new();
    let _ = writeln!(&mut buf, "Hello world").map_err(|err| {
        eprintln!(
            "[ERROR]: failed to write message to buffer because of {:?}",
            err
        );
    });
    conn.write(&buf).await?;

    let mut buf = [0; 64]; 
    loop {
        match conn.read(&mut buf).await {
            Ok(0) => {
                println!("[INFO]: client disconnected");
                break;
            },
            Ok(n) => {
                println!("The length of the buffer {n}");
            },
            Err(_) => {
                break;
            }
        }
    }
    
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    let mut stream = TcpListenerStream::new(listener);

    println!("[INFO]: server started at 127.0.0.1:3000");

    while let Some(conn) = stream.next().await {
        match conn {
            Ok(c) => {
                let _ = client(c).await;
                println!("[INFO]: client connected.");

            },
            Err(err) => {
                eprintln!("[ERROR]: client failed to connect. {:?}", err);
            }
        }
    } 

    Ok(())

    /* let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        for i in 0..10 {
            sleep(Duration::from_secs(1)).await;
            if let Err(_) = tx.send(i).await {
                println!("receiver dropped");
                return;
            }
        }
    });

    while let Some(i) = rx.recv().await {
        println!("got = {}", i);
    } */
}
