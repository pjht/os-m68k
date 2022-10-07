use std::{
    env::args,
    fs::File,
    io::{self, Write},
    path::Path,
};

const BLOCK_SIZE: u16 = 256;

fn main() {
    let mut archive = File::create("initrd").expect("Could not open initrd");
    for file_path in args().skip(1) {
        let file_path = Path::new(&file_path);
        let file_name = file_path.file_name().unwrap();
        let file_name_len =
            u8::try_from(file_name.len()).expect("File name length greater than 256 bytes");
        if file_name_len > 252 {
            panic!("File {:?} has name longer than 252 bytes", file_name);
        }
        let mut file = File::open(file_path).expect("File did not exist");
        let length = u16::try_from(file.metadata().expect("Could not get file metadata").len())
            .expect("File size greater than 64 KiB");
        let file_num_blocks = (length / BLOCK_SIZE) + if length % BLOCK_SIZE != 0 { 1 } else { 0 };
        let mut header_block = Vec::new();
        header_block.push(file_name_len);
        header_block.extend_from_slice(file_name.to_str().unwrap().as_bytes());
        header_block.push(0);
        header_block.extend_from_slice(&length.to_le_bytes());
        header_block.extend_from_slice(&file_num_blocks.to_le_bytes());
        header_block.resize(BLOCK_SIZE as usize, 0);
        archive
            .write_all(&header_block)
            .expect("Could not write to initrd");
        let bytes_written =
            io::copy(&mut file, &mut archive).expect("Could not read from file/write to initrd");
        if (bytes_written as u16 % BLOCK_SIZE) != 0 {
            let bytes_padding = (BLOCK_SIZE) - (bytes_written as u16 % BLOCK_SIZE);
            for _ in 0..bytes_padding {
                archive.write_all(&[0]).expect("Could not write to initrd");
            }
        }
    }
    archive
        .write_all(&[0; 256])
        .expect("Could not write to initrd");
}
