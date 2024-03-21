#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

mod cert;
mod tcp_server;
mod irc_server;
mod app;

fn main() {
    edit_hosts(true);
    if let Ok(cert_str) = cert::setup_cert() {
        thread::spawn(|| {
            tcp_server::run_tcp_server(cert_str).expect("Failed to start TCP server.")
        });
    };

    let (tx, rx) = channel();
    thread::spawn(move || irc_server::run_irc_server(rx));
    app::run(tx);
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

fn get_appdata_dir() -> PathBuf {
    let dir = env::var("APPDATA").expect("APPDATA environment variable not set.");
    let path = PathBuf::from(dir).join("Vermintwitch");
    if !Path::exists(&path) {
        create_dir(&path).expect("Could not create vermintwitch data dir.");
    }
    path
}