use std::fs::{create_dir_all, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;

pub fn get_filename(filepath: &String) -> &str {
    Path::new(filepath).file_name().unwrap().to_str().unwrap()
}

pub fn save_file(name: String, data: Vec<u8>) -> io::Result<()> {
    create_dir_all("downloads")?;
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("downloads/{}", name))?
        .write(&data)?;

    Ok(())
}

