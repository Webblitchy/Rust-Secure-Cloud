use std::{io};
use std::fs::{create_dir_all, OpenOptions, read};
use std::io::{Write};
use uuid::Uuid;
use crate::structs::{Company, EncryptedBox, FileNameBox};
use unidecode::unidecode;
use bincode::{serialize, deserialize, deserialize_from, serialize_into};


fn company_path(company_name: &String) -> String {
    let escaped_name = unidecode(company_name.as_str())
        .replace("/","-")
        .replace(" ", "-");
    format!("companies/{}/", escaped_name)
}

pub fn get_company(company_name: &String) -> Option<Company> {
    let company_path = company_path(company_name);
    match OpenOptions::new()
        .read(true)
        .open(company_path.to_string() + "data.bin"){
        Ok(file) => Some(deserialize_from(file).unwrap()),
        Err(_) => None
    }
}

pub fn save_company(company: &Company) -> io::Result<()> {
    let company_path = company_path(&company.name);
    create_dir_all(company_path.to_string() + "files")?;

    save_company_data(company)?;


    let empty_vec: Vec<FileNameBox> = Vec::new();
    let binary = serialize(&empty_vec).unwrap();
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(company_path.to_string() + "files.bin")?
        .write(&binary)?;

    println!("Company {} created", company.name);
    Ok(())
}

pub fn save_company_data(company: &Company) -> io::Result<()>{
    let binary = serialize(&company).unwrap();
    println!("Saving company named \"{}\"", &company.name);
    OpenOptions::new()
        .write(true)
        .create(true) // only if doesn't exist
        .open(company_path(&company.name) + "data.bin")?
        .write(&binary)?;
    Ok(())
}

pub fn save_file(company_name: &String, data: EncryptedBox, name: EncryptedBox, key: EncryptedBox) -> io::Result<()> {
    let data = serialize(&data).unwrap();
    let key = serialize(&key).unwrap();

    let uuid = Uuid::new_v4().to_string();

    OpenOptions::new()
        .write(true)
        .create(true)
        .open(company_path(company_name) + "files/" + uuid.as_str() + ".data")?
        .write(data.as_slice())?;

    OpenOptions::new()
        .write(true)
        .create(true)
        .open(company_path(company_name) + "files/" + uuid.as_str() + ".key")?
        .write(key.as_slice())?;

    let filenames = OpenOptions::new()
        .read(true)
        .open(company_path(company_name) + "files.bin")?;

    let mut filename_boxes: Vec<FileNameBox> = deserialize_from(&filenames).unwrap();
    filename_boxes.push(FileNameBox(uuid, name));

    // Reopen file otherwise cannot write
    let mut filenames = OpenOptions::new()
        .write(true)
        .open(company_path(company_name) + "files.bin")?;

    serialize_into(&mut filenames, &filename_boxes).unwrap();

    Ok(())
}

pub fn list_files(company_name: &String) -> Vec<FileNameBox> {
    let file = read(company_path(company_name) + "files.bin").unwrap();
    deserialize(&file).unwrap()
}

pub fn get_file(company_name: &String, uuid: &String) -> Result<Vec<u8>, io::Error> {
    let file_bin = &read(company_path(company_name) + "files/" + uuid.as_str() + ".data")?;
    let file : EncryptedBox = deserialize(file_bin).unwrap();
    let key_bin = &read(company_path(company_name) + "files/" + uuid.as_str() + ".key")?;
    let key : EncryptedBox = deserialize(key_bin).unwrap();
    Ok(serialize(&(file, key)).unwrap())
}