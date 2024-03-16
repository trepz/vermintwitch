use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use anyhow::Result;
use slint::{Model, SharedString, VecModel};
use winapi::ctypes::c_int;
use winapi::shared::minwindef::DWORD;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{GetKeyboardState, GetMessageW, MSG, PostThreadMessageW, RegisterHotKey, UnregisterHotKey, WM_HOTKEY, WM_QUIT};

use crate::get_appdata_dir;

slint::include_modules!();

struct Key {
    code: u32,
    name: String,
}

pub fn run(irc_sender: crossbeam_channel::Sender<String>) {
    let window = MainWindow::new().unwrap();
    let window_weak = window.as_weak();

    let (ts, tr) = channel::<DWORD>();
    let (snd, rcv) = channel::<(u32, u8)>();
    thread::spawn(move || event_loop(ts, irc_sender, rcv));
    let thread_id = tr.recv().expect("Failed to receive thread id from evt loop.");

    if let Ok(registrations) = load_registrations() {
        let strings: Vec<SharedString> = registrations.iter()
            .map(|&x| SharedString::from(win_key_lookup(x).unwrap_or("unbound".into())))
            .collect();
        registrations.iter()
            .enumerate()
            .for_each(|(i, x)| {
                snd.send((x.clone(), i as u8)).expect("Failed to fill registration channel");
            });
        window.set_keys(Rc::new(VecModel::from(strings)).into());
        unblock_event_loop(thread_id);
    }

    window.on_reg(move |slot: i32| {
        let window = window_weak.unwrap();
        match get_key_data() {
            Some(key) => {
                let snd = snd.clone();
                let mut keys: Vec<SharedString> = window.get_keys().iter().collect();
                let chopped = slot.to_le_bytes()[0];
                let owned = key.name.to_owned();
                snd.send((key.code, chopped)).expect("Channel send failed.");
                let usize_idx = usize::from(chopped);
                keys[usize_idx] = SharedString::from(owned);
                window.set_keys(Rc::new(VecModel::from(keys)).into());
                unblock_event_loop(thread_id);
            }
            None => ()
        }
    });
    window.run().unwrap();

    // Shut down evt loop
    unsafe { PostThreadMessageW(thread_id, WM_QUIT, 0, 0); }
}

fn event_loop(thread_sender: Sender<DWORD>, irc_sender: crossbeam_channel::Sender<String>, bind_receiver: Receiver<(u32, u8)>) {
    thread_sender.send(win_current_thread()).expect("Failed to identify current thread.");
    let mut registrations: Vec<u32> = vec![0; 5];
    loop {
        unsafe {
            let mut did_recv = false;
            while let Ok((code, slot)) = bind_receiver.try_recv() {
                // The slot for the binding exists, unbind it
                if registrations[slot as usize] > 0 {
                    UnregisterHotKey(null_mut(), slot as c_int);
                    did_recv = true;
                }
                // Backspace key pressed - delete and don't rebind
                if code == 0x08 {
                    registrations[slot as usize] = 0;
                } else {
                    RegisterHotKey(null_mut(), slot as c_int, 0, code);
                    registrations[slot as usize] = code;
                    did_recv = true;
                }
            }
            if did_recv {
                if let Err(_) = save_registrations(&registrations) {
                    println!("Saving keybinds failed.")
                };
            }


            let mut msg: MSG = std::mem::zeroed();
            if GetMessageW(&mut msg, null_mut(), 0, 0) == 0 {
                println!("Shutting down event loop.");
                break;
            }
            if msg.message == WM_HOTKEY {
                let txt = match msg.wParam {
                    0 => "#A",
                    1 => "#B",
                    2 => "#C",
                    3 => "#D",
                    4 => "#E",
                    _ => "???"
                };
                if let Err(_) = irc_sender.send(txt.to_string()) {
                    println!("Nobody is listening.");
                }
            }
        }
    }
}

fn unblock_event_loop(event_loop_thread_id: DWORD) {
    unsafe { PostThreadMessageW(event_loop_thread_id, 6, 0, 0); }
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
        .and_then(|(index, _)| u32::try_from(index).ok())
    {
        Some(v) => v,
        None => return None
    };

    let name = win_key_lookup(code).unwrap_or("Misc".into());
    if name == "ESC" { return None; } // Escape should cancel binding

    Some(Key { code, name })
}

fn save_registrations(reg: &Vec<u32>) -> Result<()> {
    let config = get_appdata_dir().join("binds.cfg");
    let file = File::create(config)?;
    let mut writer = BufWriter::new(file);
    for num in reg {
        write!(writer, "{}\n", num)?;
    }
    writer.flush()?;
    Ok(())
}

fn load_registrations() -> Result<Vec<u32>> {
    let config = get_appdata_dir().join("binds.cfg");
    let file = File::open(config)?;
    let reader = BufReader::new(file);
    let reg: Vec<u32> = reader
        .lines()
        .take(5)
        .map(|line| line.unwrap_or_default().parse().unwrap_or_default())
        .collect();

    Ok(reg)
}

fn win_current_thread() -> DWORD {
    unsafe {
        GetCurrentThreadId()
    }
}

fn win_key_lookup(code: u32) -> Option<String> {
    let str = match code {
        0x01 => Some("Left mouse"),
        0x02 => Some("Right mouse"),
        0x03 => Some("Control-break processing"),
        0x04 => Some("Middle mouse"),
        0x05 => Some("X1 mouse"),
        0x06 => Some("X2 mouse"),
        0x07 => Some("Reserved"),
        0x08 => Some("unbound"), // backspace key unbinds selection
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

