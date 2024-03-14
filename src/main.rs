#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

use crossbeam_channel::unbounded;

mod cert;
mod tcp_server;
mod irc_server;
mod app;

fn main() {
    edit_hosts(true);
    let api_server_running = match cert::setup_cert() {
        Ok(cert_str) => {
            thread::spawn(|| {
                tcp_server::run_tcp_server(cert_str).expect("Failed to start TCP server.")
            });
            true
        }
        Err(_) => false
    };
    let (tx, rx) = unbounded();
    thread::spawn(|| irc_server::run_irc_server(rx));
    app::run();
    edit_hosts(false);
}

fn edit_hosts(add: bool) {
    let windir = env::var("WINDIR").expect("WINDIR environment variable not set");
    let path = PathBuf::from(windir).join("System32").join("drivers").join("etc").join("hosts");
    let content = fs::read_to_string(path.clone()).unwrap();
    let mut lines = content.lines().filter_map(|line| {
        if line.contains("twitch.tv") {
            None
        } else {
            Some(line.to_string())
        }
    }).collect::<Vec<_>>();
    if add {
        lines.push("127.0.0.1 irc.chat.twitch.tv".to_string());
        lines.push("127.0.0.1 api.twitch.tv".to_string());
    }
    let new_contents = lines.join("\n");
    fs::write(path, new_contents).unwrap();
    if let Err(_) = Command::new("ipconfig").arg("/flushdns").output() {
        eprintln!("DNS flush failed.");
    }
}