use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:1883").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(mut socket: TcpStream) {
    let (mut rd, mut wr) = socket.split();
    if tokio::io::copy(&mut rd, &mut wr).await.is_err() {
        eprintln!("Failed to copy");
    }
}
