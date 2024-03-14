use winapi::um::winuser::GetKeyboardState;

slint::include_modules!();

struct Key {
    code: u16,
    name: String,
}

pub fn run() {
    let window = MainWindow::new().unwrap();
    window.on_reg(|_| {
        // Don't care about the unicode slint gives us, query windows
        match get_key_data() {
            Some(key) => println!("{} {}", key.code, key.name),
            None => println!("Unregister key")
        }
    });
    window.run().unwrap()
}


fn get_key_data() -> Option<Key> {
    // Winapi get keyboard state dumps 256 u8s representing all vkeys into a buffer
    // where a most significant bit of 1 indicates the key is currently pressed
    let mut state = [0u8; 256];
    unsafe {
        GetKeyboardState(state.as_mut_ptr());
    }

    let code = match state
        .iter()
        .enumerate()
        .find(|(_, &value)| value >= 128)
        .and_then(|(index, _)| u16::try_from(index).ok())
    {
        Some(v) => v,
        None => return None
    };

    let name = win_key_lookup(code).unwrap_or("Misc".into());

    Some(Key { code, name })
}

fn win_key_lookup(code: u16) -> Option<String> {
    let str = match code {
        0x01 => Some("Left mouse"),
        0x02 => Some("Right mouse"),
        0x03 => Some("Control-break processing"),
        0x04 => Some("Middle mouse"),
        0x05 => Some("X1 mouse"),
        0x06 => Some("X2 mouse"),
        0x07 => Some("Reserved"),
        0x08 => Some("BACKSPACE"),
        0x09 => Some("TAB"),
        0x0C => Some("CLEAR"),
        0x0D => Some("ENTER"),
        0x10 => Some("SHIFT"),
        0x11 => Some("CTRL"),
        0x12 => Some("ALT"),
        0x13 => Some("PAUSE"),
        0x14 => Some("CAPS LOCK"),
        0x1B => Some("ESC"),
        0x20 => Some("SPACEBAR"),
        0x21 => Some("PAGE UP"),
        0x22 => Some("PAGE DOWN"),
        0x23 => Some("END"),
        0x24 => Some("HOME"),
        0x25 => Some("LEFT ARROW"),
        0x26 => Some("UP ARROW"),
        0x27 => Some("RIGHT ARROW"),
        0x28 => Some("DOWN ARROW"),
        0x29 => Some("SELECT"),
        0x2A => Some("PRINT"),
        0x2B => Some("EXECUTE"),
        0x2C => Some("PRINT SCREEN"),
        0x2D => Some("INS"),
        0x2E => Some("DEL"),
        0x2F => Some("HELP"),
        0x30 => Some("0"),
        0x31 => Some("1"),
        0x32 => Some("2"),
        0x33 => Some("3"),
        0x34 => Some("4"),
        0x35 => Some("5"),
        0x36 => Some("6"),
        0x37 => Some("7"),
        0x38 => Some("8"),
        0x39 => Some("9"),
        0x41 => Some("A"),
        0x42 => Some("B"),
        0x43 => Some("C"),
        0x44 => Some("D"),
        0x45 => Some("E"),
        0x46 => Some("F"),
        0x47 => Some("G"),
        0x48 => Some("H"),
        0x49 => Some("I"),
        0x4A => Some("J"),
        0x4B => Some("K"),
        0x4C => Some("L"),
        0x4D => Some("M"),
        0x4E => Some("N"),
        0x4F => Some("O"),
        0x50 => Some("P"),
        0x51 => Some("Q"),
        0x52 => Some("R"),
        0x53 => Some("S"),
        0x54 => Some("T"),
        0x55 => Some("U"),
        0x56 => Some("V"),
        0x57 => Some("W"),
        0x58 => Some("X"),
        0x59 => Some("Y"),
        0x5A => Some("Z"),
        0x5B => Some("Left Windows"),
        0x5C => Some("Right Windows"),
        0x5D => Some("Applications"),
        0x60 => Some("Num 0"),
        0x61 => Some("Num 1"),
        0x62 => Some("Num 2"),
        0x63 => Some("Num 3"),
        0x64 => Some("Num 4"),
        0x65 => Some("Num 5"),
        0x66 => Some("Num 6"),
        0x67 => Some("Num 7"),
        0x68 => Some("Num 8"),
        0x69 => Some("Num 9"),
        0x6A => Some("Multiply"),
        0x6B => Some("Add"),
        0x6C => Some("Separator"),
        0x6D => Some("Subtract"),
        0x6E => Some("Decimal"),
        0x6F => Some("Divide"),
        0x70 => Some("F1"),
        0x71 => Some("F2"),
        0x72 => Some("F3"),
        0x73 => Some("F4"),
        0x74 => Some("F5"),
        0x75 => Some("F6"),
        0x76 => Some("F7"),
        0x77 => Some("F8"),
        0x78 => Some("F9"),
        0x79 => Some("F10"),
        0x7A => Some("F11"),
        0x7B => Some("F12"),
        0x7C => Some("F13"),
        0x7D => Some("F14"),
        0x7E => Some("F15"),
        0x7F => Some("F16"),
        0x80 => Some("F17"),
        0x81 => Some("F18"),
        0x82 => Some("F19"),
        0x83 => Some("F20"),
        0x84 => Some("F21"),
        0x85 => Some("F22"),
        0x86 => Some("F23"),
        0x87 => Some("F24"),
        0x90 => Some("NUM LOCK"),
        0x91 => Some("SCROLL LOCK"),
        0xA0 => Some("Left SHIFT"),
        0xA1 => Some("Right SHIFT"),
        0xA2 => Some("Left CONTROL"),
        0xA3 => Some("Right CONTROL"),
        0xA4 => Some("Left ALT"),
        0xA5 => Some("Right ALT"),
        0xA6 => Some("Browser Back"),
        0xA7 => Some("Browser Forward"),
        0xA8 => Some("Browser Refresh"),
        0xA9 => Some("Browser Stop"),
        0xAA => Some("Browser Search"),
        0xAB => Some("Browser Favorites"),
        0xAC => Some("Browser Start and Home"),
        0xAD => Some("Volume Mute"),
        0xAE => Some("Volume Down"),
        0xAF => Some("Volume Up"),
        0xB0 => Some("Next Track"),
        0xB1 => Some("Previous Track"),
        0xB2 => Some("Stop Media"),
        0xB3 => Some("Play/Pause Media"),
        0xB4 => Some("Start Mail"),
        0xB5 => Some("Select Media"),
        0xB6 => Some("Start Application 1"),
        0xB7 => Some("Start Application 2"),
        0xBA => Some(";"),
        0xBB => Some("+"),
        0xBC => Some(","),
        0xBD => Some("-"),
        0xBE => Some("."),
        0xBF => Some("/"),
        0xC0 => Some("`"),
        0xDB => Some("["),
        0xDC => Some("\\"),
        0xDD => Some("]"),
        0xDE => Some("'"),
        0xDF => Some("Misc"),
        0xE2 => Some("\\"),
        0xF6 => Some("Attn"),
        0xF7 => Some("CrSel"),
        0xF8 => Some("ExSel"),
        0xF9 => Some("Erase EOF"),
        0xFA => Some("Play"),
        0xFB => Some("Zoom"),
        0xFD => Some("PA1"),
        0xFE => Some("Clear"),
        _ => None
    };
    str.map(|s| s.to_string())
}

