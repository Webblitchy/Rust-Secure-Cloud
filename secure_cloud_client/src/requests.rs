use crate::authentication::build_groupkey;
use crate::crypto::{decrypt, encrypt};
use crate::files::{get_filename, save_file};
use crate::network::{read_stream, send_to_server, write_stream};
use crate::structs::RequestType::{
    AuthenticateSession, CloseConnexion, CreateCompany, DownloadFile, GetFilenames, RegenerateKey,
    UploadFile,
};
use crate::structs::{Company, EncryptedBox, FileNameBox, Key, User, ValidationType};
use crate::tui::{self, Interface, PopupType};
use bincode::{deserialize, serialize};
use dryoc::auth::Auth;
use dryoc::dryocbox::NewByteArray;
use std::fs::read;
use std::net::TcpStream;

pub fn upload_company(company: &Company, interface: &mut Interface<'_>) {
    let mut data = serialize(&company).unwrap();
    let mut stream = match send_to_server(&mut data, CreateCompany, None) {
        None => return,
        Some(stream) => stream,
    };

    match read_stream(&mut stream) {
        Ok(data) => {
            if &data == b"OK" {
                interface.set_popup("Company created on server", PopupType::Info);
            } else {
                interface.set_popup("Error when creating company", PopupType::Error);
            }
        }
        Err(e) => {
            let error = format!("Failed to receive data: {}", e);
            interface.set_popup(error.as_str(), PopupType::Error);
        }
    }
}

pub fn authenticate_session(
    interface: &mut Interface<'_>,
) -> Option<(TcpStream, Key, Key, String)> {
    let company_name =
        match tui::input_field(interface, "Your company name", &ValidationType::NotEmpty) {
            Ok(name) => name,
            Err(_) => return None,
        };
    let mut usernames: Vec<String> = Vec::new();
    let mut passwords: Vec<String> = Vec::new();
    for i in 1..3 as usize {
        let (username, password) = match tui::user_passwd_input(interface, i, false) {
            Ok((username, password)) => (username, password),
            Err(_) => return None,
        };
        usernames.push(username);
        passwords.push(password);
    }

    let mut data_to_send = serialize(&(&company_name, &usernames[0], &usernames[1])).unwrap();
    let mut stream = match send_to_server(&mut data_to_send, AuthenticateSession, None) {
        Some(stream) => stream,
        None => return None,
    };
    let data_received = read_stream(&mut stream).unwrap();
    if data_received == b"KO" {
        interface.set_popup("Bad company / usernames / passwords !", PopupType::Error);
        return None;
    }

    let (users, random, hmackey_encrypted): (Vec<User>, Vec<u8>, EncryptedBox) =
        deserialize(data_received.as_slice()).unwrap();

    let mut creds: Vec<(&User, &str)> = Vec::new();
    for i in 0..2 as usize {
        creds.push((&users[i], &passwords[i]));
    }

    let groupkey = build_groupkey(creds)?;
    let hmackey = match decrypt(&hmackey_encrypted, &groupkey) {
        Ok(hmackey) => hmackey,
        Err(_) => return None,
    };
    let mac = Auth::compute_to_vec(hmackey.clone(), &random);
    write_stream(&mut stream, mac);
    match read_stream(&mut stream) {
        Ok(data) => {
            if &data != b"KO" {
                interface.set_popup("Session authenticated", PopupType::Info);
                let enc_masterkey: EncryptedBox = deserialize(&data).unwrap();
                let masterkey: Key = match decrypt(&enc_masterkey, &groupkey) {
                    Ok(masterkey) => masterkey,
                    Err(_) => return None,
                }
                .try_into()
                .unwrap();
                return Some((stream, masterkey, hmackey.try_into().unwrap(), company_name));
            }
        }
        _ => {}
    };
    interface.set_popup("Failed to authenticate session !", PopupType::Error);
    None
}

pub fn upload_file(
    stream: TcpStream,
    masterkey: &Key,
    interface: &mut Interface<'_>,
) -> Option<TcpStream> {
    let filepath = match tui::input_field(interface, "File path", &ValidationType::ExistingFile) {
        Ok(filepath) => filepath,
        Err(_) => return Some(stream),
    };

    let file = match read(&filepath) {
        Ok(file) => file,
        Err(_) => return Some(stream),
    };

    let filename = get_filename(&filepath);
    let enc_filename = encrypt(&filename.as_bytes().to_vec(), &masterkey);

    let filekey = Key::gen().to_vec();
    let enc_filekey = encrypt(&filekey, &masterkey);

    let enc_file = encrypt(&file, filekey.as_slice().try_into().unwrap());
    let mut data = serialize(&(enc_file, enc_filename, enc_filekey)).unwrap();

    match send_to_server(&mut data, UploadFile, Some(stream)) {
        Some(mut stream) => match read_stream(&mut stream) {
            Ok(data) => {
                if data == b"OK" {
                    interface.set_popup(
                        "File successfully uploaded to the server !",
                        PopupType::Info,
                    );
                    return Some(stream);
                }
            }
            Err(_) => {}
        },
        None => {}
    }
    interface.set_popup("Failed to upload file to the server !", PopupType::Error);
    None
}

pub fn download_file(
    stream: TcpStream,
    masterkey: &Key,
    interface: &mut Interface<'_>,
) -> Option<TcpStream> {
    let filenames: Vec<FileNameBox> = match send_to_server(
        &mut Vec::new(),
        GetFilenames,
        Some(stream.try_clone().unwrap()),
    ) {
        Some(mut stream) => match read_stream(&mut stream) {
            Ok(data) => deserialize(&data).unwrap(),
            Err(_) => return None,
        },
        None => return None,
    };
    if filenames.len() == 0 {
        interface.set_popup("There is no file on the server yet !", PopupType::Info);
        return Some(stream);
    }

    let mut matching_uuid = Vec::new();
    let mut filenames_dec = Vec::new();
    for enc_filename in &filenames {
        let filename = match decrypt(&enc_filename.1, &masterkey) {
            Ok(filename) => String::from_utf8_lossy(&filename.as_slice()).to_string(),
            Err(_) => return None,
        };
        matching_uuid.push(enc_filename.0.clone()); // to get the future user choice
        filenames_dec.push(filename); // used to save file afterward
    }

    filenames_dec.push(String::from("[ Exit ]")); // quit option

    let file_i = match tui::choice_list(interface, filenames_dec.clone()) {
        Ok(option) => match option {
            Some(index) if index < matching_uuid.len() => index,
            _ => return Some(stream), // pressed esc or last choice (exit)
        },
        Err(_) => return Some(stream),
    };

    let mut chosen_file_uuid = matching_uuid[file_i].as_bytes().to_vec();

    match send_to_server(
        &mut chosen_file_uuid,
        DownloadFile,
        Some(stream.try_clone().unwrap()),
    ) {
        Some(mut stream) => {
            let data = read_stream(&mut stream).unwrap();
            if data != b"KO" {
                let (enc_file, enc_file_key): (EncryptedBox, EncryptedBox) =
                    deserialize(&data).unwrap();
                let file_key: Key = match decrypt(&enc_file_key, &masterkey) {
                    Ok(file_key) => file_key.as_slice().try_into().unwrap(),
                    Err(_) => {
                        interface.set_popup("Failed to decrypt key", PopupType::Error);
                        return None;
                    }
                };
                let file = match decrypt(&enc_file, &file_key) {
                    Ok(file) => file,
                    Err(_) => {
                        interface.set_popup("Failed to decrypt file", PopupType::Error);
                        return None;
                    }
                };
                match save_file(filenames_dec[file_i].clone(), file) {
                    Ok(_) => {
                        interface.set_popup("File successfully downloaded", PopupType::Info);
                    }
                    Err(_) => {
                        interface.set_popup("Unable to save file", PopupType::Error);
                    }
                }
                return Some(stream);
            }
        }
        None => {}
    }

    interface.set_popup("Unable to get file", PopupType::Error);
    Some(stream)
}

pub fn reupload_company(
    company: &Company,
    stream: TcpStream,
    interface: &mut Interface<'_>,
) -> Option<TcpStream> {
    let mut data = serialize(&company).unwrap();
    let mut stream =
        match send_to_server(&mut data, RegenerateKey, Some(stream.try_clone().unwrap())) {
            None => return None,
            Some(stream) => stream,
        };

    match read_stream(&mut stream) {
        Ok(data) => {
            if &data == b"OK" {
                interface.set_popup("Company key regenerated", PopupType::Info);
            } else {
                interface.set_popup("Error when rekeying company !", PopupType::Error);
            }
        }
        Err(e) => {
            let error = format!("Failed to receive data: {}", e);
            interface.set_popup(error.as_str(), PopupType::Error);
        }
    }
    None
}

pub fn disconnect(stream: TcpStream) {
    send_to_server(&mut Vec::new(), CloseConnexion, Some(stream));
}
