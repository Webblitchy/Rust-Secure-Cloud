use std::fs::read;
use std::net::TcpStream;
use bincode::{deserialize,serialize};
use dryoc::auth::Auth;
use dryoc::dryocbox::NewByteArray;
use crate::authentication::build_groupkey;
use crate::crypto::{decrypt, encrypt};
use crate::files::{get_filename, save_file};
use crate::inputs::{input_company, input_file, input_file_to_download, input_password, input_username};
use crate::structs::{Company, EncryptedBox, FileNameBox, Key, User};
use crate::structs::RequestType::{AuthenticateSession, CloseConnexion, CreateCompany, DownloadFile, GetFilenames, RegenerateKey, UploadFile};
use crate::network::{read_stream, send_to_server, write_stream};

pub fn upload_company(company: &Company) {
    let mut data = serialize(&company).unwrap();
    let mut stream = match send_to_server(&mut data, CreateCompany, None) {
        None => return,
        Some(stream) => stream
    };

    match read_stream(&mut stream) {
        Ok(data) => {
            if &data == b"OK" {
                println!("=> Company created on server");
            }
            else {
                eprintln!("Error when creating company");
            }
        },
        Err(e) => {
            eprintln!("Failed to receive data: {}", e);
        }
    }
}

pub fn authenticate_session() -> Option<(TcpStream, Key, Key, String)> {
    let company_name = input_company();
    let mut usernames : Vec<String> = Vec::new();
    let mut passwords: Vec<String> = Vec::new();
    for i in 0..2 as usize {
        println!("For user no {}", i + 1);
        usernames.push(input_username());
        passwords.push(input_password(false));
    }

    let mut data_to_send = serialize(&(&company_name, &usernames[0], &usernames[1])).unwrap();
    let mut stream =  match send_to_server(&mut data_to_send, AuthenticateSession, None) {
        Some(stream) => stream,
        None => return None,
    };
    let data_received = read_stream(&mut stream).unwrap();
    if data_received == b"KO" {
        eprintln!("Bad company / usernames / passwords");
        return None;
    }

    let (users, random, hmackey_encrypted): (Vec<User>, Vec<u8>, EncryptedBox) = deserialize(data_received.as_slice()).unwrap();

    let mut creds : Vec<(&User, &str)> = Vec::new();
    for i in 0..2 as usize {
        creds.push((&users[i], &passwords[i]));
    }

    let groupkey = build_groupkey(creds)?;
    let hmackey = match decrypt(&hmackey_encrypted, &groupkey) {
        Ok(hmackey) => hmackey,
        Err(_) => return None
    };
    let mac = Auth::compute_to_vec(hmackey.clone(), &random);
    write_stream(&mut stream, mac);
    match read_stream(&mut stream) {
        Ok(data) => {
            if &data != b"KO" {
                println!("=> Session authenticated !");
                let enc_masterkey: EncryptedBox = deserialize(&data).unwrap();
                let masterkey: Key = match decrypt(&enc_masterkey, &groupkey) {
                    Ok(masterkey) => masterkey,
                    Err(_) => return None
                }.try_into().unwrap();
                return Some((stream, masterkey, hmackey.try_into().unwrap(), company_name));
            }
        },
        _ => {}
    };
    eprintln!("Failed to authenticate session");
    None
}

pub fn upload_file(stream: TcpStream, masterkey: &Key) -> Option<TcpStream> {
    let filepath = input_file();

    if filepath == "" { // user pressed enter
        return Some(stream);
    }

    let file = read(&filepath).unwrap();

    let filename = get_filename(&filepath);
    let enc_filename = encrypt(&filename.as_bytes().to_vec(), &masterkey);

    let filekey = Key::gen().to_vec();
    let enc_filekey = encrypt(&filekey, &masterkey);

    let enc_file = encrypt(&file, filekey.as_slice().try_into().unwrap());
    let mut data = serialize(&(enc_file, enc_filename, enc_filekey)).unwrap();

    match send_to_server(&mut data, UploadFile, Some(stream)) {
        Some(mut stream) => {
            match read_stream(&mut stream) {
                Ok(data) => {
                    if data == b"OK" {
                        println!("=> File successfully uploaded to the server !");
                        return Some(stream);
                    }
                }
                Err(_) => {}
            }
        }
        None => {}
    }
    eprintln!("Failed to upload file to the server");
    None
}

pub fn download_file(stream: TcpStream, masterkey: &Key) -> Option<TcpStream> {

    let filenames : Vec<FileNameBox> = match send_to_server(&mut Vec::new(), GetFilenames, Some(stream.try_clone().unwrap())) {
        Some(mut stream) => {
            match read_stream(&mut stream) {
                Ok(data) => deserialize(&data).unwrap(),
                Err(_) => return None
            }
        }
        None => return None
    };
    if filenames.len() == 0 {
        println!("There is no file on server yet");
        return Some(stream);
    }

    println!("== FILES ON THE SERVER ==");
    let mut i = 1;
    let mut matching_uuid = Vec::new();
    let mut filenames_dec = Vec::new();
    for enc_filename in &filenames {
        let filename : String = match decrypt(&enc_filename.1, &masterkey) {
            Ok(filename) => String::from_utf8_lossy(&filename.as_slice()).to_string(),
            Err(_) => return None
        };
        println!("[{}] {}", i, filename);
        matching_uuid.push(enc_filename.0.clone()); // to get the future user choice
        filenames_dec.push(filename); // used to save file afterward
        i += 1;
    }

    let mut file_i = input_file_to_download(matching_uuid.len());
    if file_i == 0 { // user pressed enter
        return Some(stream);
    } else {
        file_i -= 1; // convert to index
    }
    let mut chosen_file_uuid = matching_uuid[file_i].as_bytes().to_vec();

     match send_to_server(&mut chosen_file_uuid, DownloadFile, Some(stream.try_clone().unwrap())) {
        Some(mut stream) => {
            let data = read_stream(&mut stream).unwrap();
            if data != b"KO" {
                let (enc_file, enc_file_key) : (EncryptedBox, EncryptedBox) = deserialize(&data).unwrap();
                let file_key: Key = match decrypt(&enc_file_key, &masterkey) {
                    Ok(file_key) => file_key.as_slice().try_into().unwrap(),
                    Err(_) => {
                        eprintln!("Failed to decrypt key");
                        return None;
                    }
                };
                let file = match decrypt(&enc_file, &file_key) {
                    Ok(file) => file,
                    Err(_) => {
                        eprintln!("Failed to decrypt file");
                        return None;
                    }
                };
                match save_file(&filenames_dec[file_i], file) {
                    Ok(_) => {
                        println!("=> File successfully downloaded");
                    }
                    Err(_) => {
                        eprintln!("Unable to save file");
                    }
                }
                return Some(stream);
            }
        },
        None => {}
    }
    eprintln!("Unable to get file");
    Some(stream)
}

pub fn reupload_company(company: &Company, stream: TcpStream) -> Option<TcpStream> {
    let mut data = serialize(&company).unwrap();
    let mut stream = match send_to_server(&mut data, RegenerateKey, Some(stream.try_clone().unwrap())) {
        None => return None,
        Some(stream) => stream
    };

    match read_stream(&mut stream) {
        Ok(data) => {
            if &data == b"OK" {
                println!("=> Company key regenerated");
            }
            else {
                eprintln!("Error when rekeying company");
            }
        },
        Err(e) => {
            eprintln!("Failed to receive data: {}", e);
        }
    }
    None
}

pub fn disconnect(stream: TcpStream) {
    send_to_server(&mut Vec::new(), CloseConnexion, Some(stream));
}