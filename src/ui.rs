use std::collections::HashMap;

use slint::SharedString;

slint::include_modules!();

pub fn run_ui() {
    let window = MainWindow::new().unwrap();
    let map = unicode_map();
    window.on_reg(move |key: SharedString| {
        match map.get(&key.as_str().to_string()) {
            Some((code, name)) => println!("Got {} with name {}", code, name),
            _ => println!("No key match")
        }
    });
    window.run().unwrap()
}

fn unicode_map() -> HashMap<String, (u32, String)> {
    let mut map: HashMap<String, (u32, String)> = HashMap::new();

    map.insert("\u{F704}".into(), (0x41, "F1".into()));
    map.insert("\u{F705}".into(), (0x42, "F2".into()));

    map
}