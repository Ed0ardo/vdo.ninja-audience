#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// --- Import necessary modules ---
use tauri::{Manager};
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use rand::Rng;
use rand::distributions::Distribution;

// --- Configuration Structure ---
/// Defines the structure for the application configuration.
#[derive(Serialize, Deserialize)]
struct AppConfig {
    vdo_ninja_url: String,
}

// --- Helper Functions ---

/// Returns the path to the configuration file.
/// The file is located in the same directory as the executable.
fn get_config_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let exe_path = app_handle.path_resolver().resolve_resource("").unwrap().parent().unwrap().to_path_buf();
    exe_path.join("config.json")
}

/// Returns the path to the encryption key file.
/// The file is located in the same directory as the executable.
fn get_key_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let exe_path = app_handle.path_resolver().resolve_resource("").unwrap().parent().unwrap().to_path_buf();
    exe_path.join("encryption.key")
}

/// Generates a new encryption key and saves it to a file.
fn generate_new_encryption_key(app_handle: &tauri::AppHandle) -> String {
    let key = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();
    fs::write(get_key_path(app_handle), &key).unwrap();
    key
}

/// Loads the encryption key from a file or generates a new one if it doesn't exist.
fn load_or_generate_encryption_key(app_handle: &tauri::AppHandle) -> String {
    match fs::read_to_string(get_key_path(app_handle)) {
        Ok(key) => key,
        Err(_) => generate_new_encryption_key(app_handle),
    }
}

/// Saves the VDO.Ninja URL to the configuration file, encrypting it first.
fn save_encrypted_url_to_config(app_handle: &tauri::AppHandle, url: &str) {
    let key = load_or_generate_encryption_key(app_handle);
    let mc = new_magic_crypt!(&key, 256);
    let encrypted_url = mc.encrypt_str_to_base64(url);
    let config = AppConfig { vdo_ninja_url: encrypted_url };
    let json = serde_json::to_string(&config).unwrap();
    fs::write(get_config_path(app_handle), json).unwrap();
}

/// Loads the VDO.Ninja URL from the configuration file, decrypting it first.
fn load_decrypted_url_from_config(app_handle: &tauri::AppHandle) -> Option<String> {
    if let Ok(data) = fs::read_to_string(get_config_path(app_handle)) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
            let key = load_or_generate_encryption_key(app_handle);
            let mc = new_magic_crypt!(&key, 256);
            if let Ok(decrypted_url) = mc.decrypt_base64_to_string(&config.vdo_ninja_url) {
                return Some(decrypted_url);
            }
        }
    }
    None
}

/// Generates a new secure VDO.Ninja URL with a random push ID and audience password.
fn generate_random_secure_url() -> String {
    use rand::thread_rng;
    use rand::distributions::Alphanumeric;
    use rand::seq::SliceRandom;

    let mut rng = thread_rng();

    // Genera push_id usando la distribuzione Alphanumeric
    let push_id: String = Alphanumeric
        .sample_iter(&mut rng)
        .take(8)
        .map(char::from)
        .collect();

    let uppercase = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lowercase = b"abcdefghijklmnopqrstuvwxyz";
    let digits = b"0123456789";
    let special = b"!@#$%^&*";

    let mut password_bytes = Vec::with_capacity(16);

    // Ensure at least one character of each type:
    password_bytes.push(*uppercase.choose(&mut rng).unwrap());
    password_bytes.push(*lowercase.choose(&mut rng).unwrap());
    password_bytes.push(*digits.choose(&mut rng).unwrap());
    password_bytes.push(*special.choose(&mut rng).unwrap());

    let mut all_chars_bytes = Vec::new();
    all_chars_bytes.extend_from_slice(uppercase);
    all_chars_bytes.extend_from_slice(lowercase);
    all_chars_bytes.extend_from_slice(digits);
    all_chars_bytes.extend_from_slice(special);

    // Add the remaining random characters from all sets
    for _ in 0..(16 - 4) {
        password_bytes.push(*all_chars_bytes.choose(&mut rng).unwrap());
    }

    // Shuffle the password to randomize order
    password_bytes.shuffle(&mut rng);

    let audience_pass = String::from_utf8(password_bytes).unwrap();

    format!("https://vdo.ninja/?push={}&audience={}", push_id, audience_pass)
}

/// Validates the audience password based on a set of rules.
fn validate_audience_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long.".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter.".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter.".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one digit.".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character (!@#$%^&*).".to_string());
    }

    Ok(())
}

/// Sets the VDO.Ninja URL and saves it to the configuration.
#[tauri::command]
fn set_and_save_vdo_ninja_link(window: tauri::Window, push_id: String, audience: String) -> Result<(), String> {
    let app_handle = window.app_handle();
    let mut manual_url = format!("https://vdo.ninja/?push={}", push_id);
    if !audience.is_empty() {
        if let Err(e) = validate_audience_password(&audience) {
            return Err(e);
        }
        manual_url.push_str(&format!("&audience={}", audience));
    }
    save_encrypted_url_to_config(&app_handle, &manual_url);
    println!("Saved manual URL: {}", manual_url);

    // Emit an event to notify the main window that the URL has been updated
    if let Some(main_window) = app_handle.get_window("main") {
        main_window.emit("url-updated", &manual_url).unwrap();
    }

    Ok(())
}

// --- Tauri Commands (callable from JS) ---

/// Command to generate a new random VDO.Ninja link.
#[tauri::command]
fn generate_new_random_link(window: tauri::Window) {
    let app_handle = window.app_handle();
    let full_url = generate_random_secure_url();
    
    // Save the URL directly without parsing (since we don't use the parsed values)
    save_encrypted_url_to_config(&app_handle, &full_url);
    println!("Generated and saved random URL: {}", full_url);
    
    // Emit an event to notify the main window that the URL has been updated
    if let Some(main_window) = app_handle.get_window("main") {
        main_window.emit("url-updated", &full_url).unwrap();
    }
}



/// Command to load the decrypted URL from config.
#[tauri::command]
fn load_decrypted_url_from_config_command(app_handle: tauri::AppHandle) -> Option<String> {
    load_decrypted_url_from_config(&app_handle)
}

// --- Main Application Setup ---
fn main() {
    // --- Tauri Application Builder ---
    tauri::Builder::default()
        .setup(|app| {
            let _main_window = app.get_window("main").unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![generate_new_random_link, load_decrypted_url_from_config_command, set_and_save_vdo_ninja_link])
        .run(tauri::generate_context!()) 
        .expect("error while running tauri application");
}
