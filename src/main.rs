use std::io::{self, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

mod vbios;
use crate::vbios::{VBios, VBiosBuilder};

const FLASH_FLAG: [u8; 5] = [0x5A, 0x5A, 0xA5, 0xA5, 0x01];
const DRIVER_FLAG: [u8; 5] = [0x5A, 0xA5, 0x5A, 0xA5, 0x01];

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("USAGE:");
        println!("    vbios.exe <SUBCOMMAND>");
        println!("SUBCOMMAND:");
        println!("    generate  -> generate executable");
        println!("    update    -> update / flash bios");
        return Ok(());
    }
    if args[1] == "generate" {
        println!("...generate");
        let vbios = VBiosBuilder::new("./target/release/vbios.exe")
            .add_bin("./target/release/input.exe", FLASH_FLAG.into())
            .add_bin("./target/release/insttool64.exe", DRIVER_FLAG.into())
            .build()?;
        vbios.write_all("./target/release/vbiospack.exe")?;
    } else if args[1] == "update" {
        println!("...update");
        let vbios = VBios::from(&args[0]);
        let flash_index = vbios.find_flag(FLASH_FLAG.to_vec());
        let driver_index = vbios.find_flag(DRIVER_FLAG.to_vec());

        let tmp_dir = std::env::temp_dir().join("vbios");
        let flash_path = tmp_dir.join("input.exe");
        let driver_path = tmp_dir.join("insttool64.exe");
        std::fs::create_dir_all(&tmp_dir)?;

        vbios.export_bin(&flash_path, flash_index + 9..driver_index)?;
        vbios.export_bin(&driver_path, driver_index + 9..vbios.size())?;

        println!("...initializing");
        print!("driver: {}", run_process(driver_path));
        println!("......done");
        println!("...flashing");
        print!("flash: {}", run_process(flash_path));
        println!("......done");
        std::fs::remove_dir_all(tmp_dir)?;
    } else {
        println!("USAGE:");
        println!("    vbios.exe <SUBCOMMAND>");
        println!("SUBCOMMAND:");
        println!("    generate  -> generate executable");
        println!("    update    -> update / flash bios");
        return Ok(());
    }
    println!("...finish");
    Ok(())
}

fn run_process(cmd: std::path::PathBuf) -> String {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let t = thread::spawn(move || {
        let output = Command::new(cmd)
            .output()
            .expect("failed to execute process");
        tx.send(output).unwrap();
        stop_clone.store(true, Ordering::Relaxed);
    });
    let mut loading_char = vec!["\\", "|", "/", "-"].into_iter().cycle();
    while !stop.load(Ordering::Relaxed) {
        thread::sleep(time::Duration::from_millis(100));
        print!("please wait {}\r", loading_char.next().unwrap());
        std::io::stdout().flush().unwrap();
    }
    print!("{:24 }\r", "");
    t.join().unwrap();
    String::from_utf8_lossy(&rx.recv().unwrap().stdout).to_string()
}
