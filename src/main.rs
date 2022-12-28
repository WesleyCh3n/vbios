use std::fs::OpenOptions;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    {
        let mut output_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("./target/debug/vbiospack.exe")?;
        let mut content = std::fs::read("./target/debug/vbios.exe")?;
        let size: [u8; 8] = content.len().to_le_bytes();
        println!("len: {:?}", content.len());
        for i in size {
            print!("{:#04x} ", i);
        }
        println!("");
        let mut flag = vec![0x5A, 0x5A, 0xA5, 0xA5, 0x01];
        flag.extend_from_slice(&size[..4]);
        content.extend_from_slice(&flag);
        output_file.write_all(&content)?;
    }
    {
        let content = std::fs::read("./target/debug/vbiospack.exe")?;
        for i in &content[content.len() - 4..] {
            print!("{:#04x} ", i);
        }
        println!("");
        let result = i32::from_le_bytes(
            (&content[content.len() - 4..]).try_into().unwrap(),
        );
        println!("result: {}", result);
        let index = content
            .windows(5)
            .enumerate()
            .position(|(i, window)| {
                window == &[0x5A, 0x5A, 0xA5, 0xA5, 0x01]
                    && i == i32::from_le_bytes(
                        (&content[i + 5..i + 9]).try_into().unwrap(),
                    ) as usize
            })
            .unwrap();
        println!("index: {}", index);
    }
    Ok(())
}
