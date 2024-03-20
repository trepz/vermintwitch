use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

use crossbeam_channel::Receiver;

pub fn run_irc_server(receiver: Receiver<String>) {
    let rx = receiver.clone();
    thread::spawn(|| channel_drain(rx));
    let listener = TcpListener::bind("127.0.0.1:6667").unwrap();
    while let Ok((stream, _)) = listener.accept() {
        let rx = receiver.clone();
        thread::spawn(move || handle_client(stream, rx));
    }
}

fn handle_client(mut stream: TcpStream, chan: Receiver<String>) {
    let mut rat_num = 0;
    loop {
        let vote = match chan.recv() {
            Ok(v) => v,
            _ => break,
        };
        let msg = format!(
            ":Rat{0}!rat{0}@user.tmi.twitch.tv PRIVMSG #rat :{1}\r\n",
            rat_num, vote
        );
        if let Err(_) = stream.write_all(msg.as_bytes()) {
            break;
        };
        rat_num = rat_num + 1;
        if rat_num > 50 {
            rat_num = 0;
        }
    }
}

fn channel_drain(chan: Receiver<String>) {
    // Just drain this channel forever so we don't blow up the PC if the user holds a hotkey
    // forever without being connected.
    loop {
        chan.recv().unwrap();
    }
}