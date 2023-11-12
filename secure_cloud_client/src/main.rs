use crate::creation::{create_company, rekey_company};
use crate::requests::{
    authenticate_session, disconnect, download_file, reupload_company, upload_company, upload_file,
};
use crate::structs::Key;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use shamirsecretsharing::hazmat::KEY_SIZE;
use std::io;
use std::net::TcpStream;
use tui::Interface;

mod authentication;
mod creation;
mod crypto;
mod files;
mod inputs;
mod network;
mod requests;
mod shamir;
mod structs;
mod tui;

fn main() -> io::Result<()> {
    let mut stream: Option<TcpStream> = None;
    let mut masterkey: Key = [0; KEY_SIZE];
    let mut company_name = String::new();
    let mut hmackey: Key = [0; KEY_SIZE];

    // Set the UI
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut interface = Interface::new(Terminal::new(backend)?);

    // tui::input_field(&mut term, "Enter..")?;
    let choices = vec![
        String::from("Download a file"),
        String::from("Upload a file"),
        String::from("Regenerate key"),
        String::from("Create a company"),
        String::from("Close program"),
    ];

    loop {
        let option = match tui::choice_list(&mut interface, choices.clone()) {
            Ok(val) => {
                match val {
                    Some(selection) => selection,
                    None => 4, // quit
                }
            }
            Err(_) => 4,
        };

        if stream.is_none() && option <= 2 {
            match authenticate_session(&mut interface) {
                Some((s, m, hmac, name)) => {
                    stream = Some(s);
                    masterkey = m;
                    hmackey = hmac;
                    company_name = name;
                }
                None => continue,
            }
        }
        match option {
            0 => {
                // DOWNLOAD FILE
                stream = download_file(stream.unwrap(), &masterkey, &mut interface);
            }
            1 => {
                // UPLOAD FILE
                stream = upload_file(stream.unwrap(), &masterkey, &mut interface);
            }
            2 => {
                // REGENERATE KEYS
                let company = rekey_company(&masterkey, &hmackey, &company_name, &mut interface);
                stream = reupload_company(&company, stream.unwrap(), &mut interface);
            }
            3 => {
                // CREATE COMPANY
                let company = create_company(&mut interface);
                upload_company(&company, &mut interface);
            }
            4 => {
                // CLOSING PROGRAM
                if !stream.is_none() {
                    disconnect(stream.unwrap());
                }
                break;
            }
            _ => {} // not reachable
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(interface.term.backend_mut(), LeaveAlternateScreen,)?;
    Ok(())
}
