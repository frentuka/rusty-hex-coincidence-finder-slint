use std::fs::File;
use std::io::{self, Read, Write};

pub fn read_binary_file(path: &str) -> io::Result<Vec<u8>> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {}", path, e));
    let mut buffer: Vec<u8> = Vec::new();
    file.unwrap().read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn modify_file(file: Vec<u8>, position: u32, modification: Vec<u8>) -> Vec<u8> {
    let old_slice = &file.clone()[position as usize..(position + modification.len() as u32) as usize];
    let mut file = file;
    for i in 0..modification.len() {
        file[(position + i as u32) as usize] = modification[i];
    }
    
    println!("New value into file: {:?}", modification);
    println!("where there were {old_slice:?}");
    
    file
}

pub fn store_file(filedata: Vec<u8>, path: &str) -> io::Result<()> {
    let mut file = File::create(path).map_err(|e| format!("failed to create {}: {}", path, e)).unwrap();
    file.write_all(&filedata)?;
    Ok(())
}