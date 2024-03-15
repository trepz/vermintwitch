use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use anyhow::Result;
use native_tls::{Identity, TlsAcceptor};

pub fn run_tcp_server(key: Vec<u8>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:443")?;
    let identity = Identity::from_pkcs12(&key, "Testing")?;

    let build = TlsAcceptor::builder(identity).build()?;
    let acceptor = TlsAcceptor::from(build);

    while let Ok((stream, _)) = listener.accept() {
        let acc = acceptor.clone();
        thread::spawn(move || handle_connection(stream, acc));
    };

    Ok(())
}

fn handle_connection(stream: TcpStream, acceptor: TlsAcceptor) {
    match acceptor.accept(stream) {
        Ok(mut tls_stream) => {
            let response = "HTTP/1.1 200 OK\r\n\
                Content-Length: 56\r\n\
                Content-Type: application/json; charset=utf-8\r\n\r\n\
                {\"data\":[{\"id\":\"420\",\"login\":\"rat\",\"user_login\":\"rat\"}]}";
            tls_stream.write_all(response.as_bytes()).unwrap_or_else(|_| {
                eprintln!("Stream write failed");
            });
            tls_stream.flush().unwrap_or_else(|_| {
                eprintln!("Stream flush failed");
            });
            println!("Rat authenticated");
        }
        Err(e) => {
            eprintln!("TLS handshake failed: {}", e);
        }
    }
}