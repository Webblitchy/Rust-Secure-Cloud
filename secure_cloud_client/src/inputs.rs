use ratatui::style::{Color, Style};
use read_input::{prelude::input, InputBuild, InputConstraints};
use rpassword::prompt_password;
use std::env;
use std::path::Path;
use tui_textarea::TextArea;
use zxcvbn::zxcvbn;

use crate::structs::ValidationType;

fn check_password_strength(password: &String) -> bool {
    if password.len() < 12 || password.len() > 64 {
        return false;
    }

    if zxcvbn(password, &[]).unwrap().score() < 3 {
        return false;
    }

    true
}

pub fn validate_input(textarea: &mut TextArea, validation_type: &ValidationType) -> bool {
    let input = &textarea.lines()[0];
    let is_valid = match validation_type {
        ValidationType::NotEmpty => !input.is_empty(),
        ValidationType::Password => check_password_strength(input),
        ValidationType::NbMinUser => input.parse::<u8>().is_ok_and(|nb| nb > 1),
        ValidationType::ExistingFile => Path::new(input).is_file(),
    };
    let font_color;
    if is_valid {
        font_color = Color::LightGreen;
    } else {
        font_color = Color::LightRed;
    }
    textarea.set_style(Style::default().fg(font_color));
    is_valid
}

pub fn input_nb_users() -> u8 {
    input::<u8>()
        .msg("Enter number of users: ")
        .min(2)
        .err("The number of user must be between 2 and 255")
        .get()
}

pub fn input_username() -> String {
    input::<String>().msg("Enter username : ").get()
}

pub fn input_password(password_check_needed: bool) -> String {
    let bad_password_msg = "Password must be between 12 and 64 characters and must be strong";

    loop {
        let password = match prompt_password("Enter password: ") {
            Ok(password) => password,
            Err(_) => {
                println!("> Your console doesn't support hidden input");
                println!("  (please hide the password manually)");
                break;
            }
        };

        if password_check_needed {
            if !check_password_strength(&password) {
                println!("{}", bad_password_msg);
                continue;
            }

            let password_confirm = prompt_password("Confirm password: ").unwrap();

            if password != password_confirm {
                println!("Passwords do not match");
                continue;
            }
        }

        return password;
    }

    // if user doesn't support hidden input (ex: in CLion's console)
    input::<String>()
        .msg("Enter password: ")
        .add_test(move |password| !password_check_needed || check_password_strength(password))
        .err(bad_password_msg)
        .get()
}

pub fn input_company() -> String {
    input::<String>()
        .msg("Enter company name: ")
        .add_test(|name| name.len() > 0)
        .get()
}

pub fn input_option() -> u8 {
    input::<u8>()
        .min(1)
        .max(5)
        .err(format!("Choose between 1 and 5"))
        .msg("Choose the option with the corresponding number : ")
        .get()
}

pub fn input_file() -> String {
    input::<String>()
        .msg("Enter the path of the file you want to upload (cancel with ENTER) : ")
        .add_test(|s| Path::new(s).is_file())
        .default("".to_string()) // if no file entered, it will return an empty string
        .err(format!(
            "This file doesn't exist\nEnter a path relative from {}/ or absolute",
            env::current_dir().unwrap().display()
        ))
        .get()
}

pub fn input_file_to_download(max: usize) -> usize {
    input::<usize>()
        .msg("Choose the file with the corresponding number (cancel with ENTER): ")
        .min(1)
        .max(max)
        .default(0) // if user just press enter, it will return 0
        .err(format!("Enter a number between 1 and {}", max))
        .get()
}
