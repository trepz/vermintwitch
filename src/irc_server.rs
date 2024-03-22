use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;

pub fn run_irc_server(receiver: Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();
    let streams: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(vec![]));
    let streams_copy = Arc::clone(&streams);

    // Write to all streams when channel receives a vote
    thread::spawn(move || {
        let mut rat_num = 0;
        loop {
            let vote = receiver.recv().expect("Irc receiver failed.");
            let mut s = streams_copy.lock().unwrap();
            s.retain(|mut stream| {
                let msg = format!(":Rat{0}!rat{0}@user.tmi.twitch.tv PRIVMSG #rat :{1}\r\n", rat_num, vote);
                if let Err(_) = stream.write_all(msg.as_bytes()) {
                    return false;
                };
                rat_num = rat_num + 1;
                if rat_num > 50 {
                    rat_num = 0;
                };
                true
            })
        }
    });

    while let Ok((stream, _)) = listener.accept() {
        streams.lock().unwrap().push(stream);
    }
}