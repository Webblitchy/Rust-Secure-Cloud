use std::io::{Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use dryoc::auth::Auth;
use dryoc::rng::randombytes_buf;
use bincode::{serialize, deserialize};
use crate::files::{get_company, get_file, list_files, save_company, save_company_data, save_file};
use crate::network::{read_stream, shutdown_stream, write_stream};
use crate::structs::{Company, EncryptedBox, RequestType};

mod files;
mod structs;
mod network;

const SERVER_ADDR: &str = "127.0.0.1:1234";

fn main() {
    run_server();
}

fn run_server() {
    let listener = match TcpListener::bind(SERVER_ADDR) {
        Ok(listener) => listener,
        Err(e) => {
            println!("{e}\nQUITTING");
            return;
        }
    };

    println!("Server running");
    for stream in listener.incoming() {
        spawn(move || { // gère chacune des connexions dans un thread
            match stream {
                Ok(stream) => {
                    println!("----------------------------");
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    handle_client(stream); // connection succeeded
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });
    }
    // close the socket server
    drop(listener);
}



fn handle_client(mut stream: TcpStream) {
    let mut company = Company::empty_company();
    loop {
        match read_stream(&mut stream) {
            Ok(data) => {
                if data.len() == 0 {
                    println!("Client disconnected");
                    shutdown_stream(&mut stream);
                    return;
                }
                let request_type = match data[data.len() - 1].try_into() {
                    Ok(request_type) => request_type,
                    Err(_) => {
                        eprintln!("Bad request, disconnect client");
                        shutdown_stream(&mut stream);
                        return;
                    }
                };

                let data = &data[0..data.len() - 1];
                match request_type {
                    RequestType::CloseConnexion => {
                        println!("Client closed connexion");
                        shutdown_stream(&mut stream);
                        return;
                    },
                    RequestType::CreateCompany => {
                        let company: Company = deserialize(data).unwrap();
                        if get_company(&company.name).is_none() {
                            if save_company(&company).is_ok() { // && made concurrency issues
                                stream.write(b"OK").unwrap();
                                continue;
                            }
                        }
                        stream.write(b"KO").unwrap();
                    },
                    RequestType::AuthenticateSession => {
                        let (company_name, user1, user2): (String, String, String) = deserialize(data).unwrap();
                        company = match get_company(&company_name) {
                            Some(company) => company,
                            None => {
                                eprintln!("Error: Company not found");
                                write_stream(&mut stream, vec![0]); // sending error
                                continue;
                            }
                        };


                        let mut users = Vec::new();
                        for user in [user1, user2] {
                            let u = match company.find_user(user) {
                                Some(user) => user,
                                None => break
                            };
                            users.push(u);
                        }

                        if users.len() != 2 {
                            eprintln!("Bad username");
                            stream.write(b"KO").unwrap();
                            continue;
                        }

                        let random = randombytes_buf(64);
                        let data_to_send = (users, &random, company.hmackey_encrypted);
                        let data = serialize(&data_to_send).unwrap();
                        write_stream(&mut stream, data);
                        let received_mac = read_stream(&mut stream).unwrap();
                        if received_mac.len() != 32 {
                            eprintln!("Bad MAC");
                            continue;
                        }
                        match Auth::compute_and_verify(&received_mac, company.hmackey, &random) {
                            Ok(_) => {
                                println!("Session authenticated");
                                let buffer = serialize(&company.masterkey_encrypted).unwrap();
                                write_stream(&mut stream, buffer);
                            },
                            Err(_) => {
                                println!("Authentication failed");
                                stream.write(b"KO").unwrap(); // if mac is not valid
                            },
                        };
                    }
                    RequestType::SaveFile => {
                        let (file, filename, key): (EncryptedBox, EncryptedBox, EncryptedBox) = deserialize(data).unwrap();
                        match save_file(&company.name, file, filename, key) {
                            Ok(_) => stream.write(b"OK").unwrap(),
                            Err(_) => stream.write(b"KO").unwrap()
                        };
                        println!("File saved on server");
                    }
                    RequestType::GetFilenames => {
                        let files = list_files(&company.name);
                        let binary = serialize(&files).unwrap();
                        write_stream(&mut stream, binary);
                    }
                    RequestType::SendFile => {
                        let uuid = String::from_utf8_lossy(data).to_string();
                        match get_file(&company.name, &uuid) {
                            Ok(file) => {
                                write_stream(&mut stream, file);
                            },
                            Err(_) => {
                                eprintln!("Failed to load file");
                                stream.write(b"KO").unwrap();
                            }
                        }
                    },
                    RequestType::RegenerateKey => {
                        company = deserialize(data).unwrap(); // TODO
                        println!("{:?}", company);
                        if save_company_data(&company).is_ok() {
                            stream.write(b"OK").unwrap();
                            continue;
                        }
                        eprintln!("Failed to save company");
                        stream.write(b"KO").unwrap();
                    }
                }
            },
            Err(_) => {
                eprintln!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                shutdown_stream(&mut stream);
            }
        }
    }
}