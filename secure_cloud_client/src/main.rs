use std::net::TcpStream;
use shamirsecretsharing::hazmat::KEY_SIZE;
use crate::requests::{upload_company, authenticate_session, download_file, upload_file, disconnect, reupload_company};
use crate::creation::{create_company, rekey_company};
use crate::inputs::input_option;
use crate::structs::{Key};

mod creation;
mod structs;
mod authentication;
mod crypto;
mod shamir;
mod requests;
mod network;
mod inputs;
mod files;


fn main() {
    let mut stream : Option<TcpStream> = None;
    let mut masterkey : Key = [0; KEY_SIZE];
    let mut company_name = String::new();
    let mut hmackey : Key = [0; KEY_SIZE];

    print!("============== SECURE CLOUD ==============");
    loop {
        println!("\n[1] Download a file");
        println!("[2] Upload a file");
        println!("[3] Regenerate keys");
        println!("[4] Create a company");
        println!("[5] Close program");


        let option = input_option();

        if stream.is_none() && option <= 3 {
            println!("\nAUTHENTICATE SESSION");
            match authenticate_session() {
                Some((s, m, hmac, name)) => {
                    stream = Some(s);
                    masterkey = m;
                    hmackey = hmac;
                    company_name = name;
                },
                None => continue,
            }
        }
        match option {
            1 => {
                println!("\nDOWNLOAD FILE");
                stream = download_file(stream.unwrap(), &masterkey);
            },
            2 => {
                println!("\nUPLOAD FILE");
                stream = upload_file(stream.unwrap(), &masterkey);
            },
            3 => {
                println!("\nREGENERATE KEYS");
                let company = rekey_company(&masterkey, &hmackey, &company_name);
                stream = reupload_company(&company, stream.unwrap());
            },
            4 => {
                println!("\nCREATE COMPANY");
                let company = create_company();
                upload_company(&company);
            },
            5 => {
                println!("\nCLOSING PROGRAM");
                if !stream.is_none() {
                    disconnect(stream.unwrap());
                }
                break;
            }
            _ => {} // not reachable
        }
    }



}
