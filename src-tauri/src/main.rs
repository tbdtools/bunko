// Note: windows_subsystem = "windows" removed to ensure CLI works properly

use std::path::Path;
use serde_json;
use tauri::{self, Manager};
use tauri_plugin_cli::CliExt;

use bunko_lib::{notebook, vault};
use bunko_lib::{Note, Vault};

mod tui;

#[tauri::command]
async fn init_notebook(path: String) -> Result<(), String> {
    notebook::init_notebook(Path::new(&path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_note(notebook_path: String, title: String, content: String) -> Result<Note, String> {
    notebook::add_note(Path::new(&notebook_path), &title, &content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_notes(notebook_path: String) -> Result<Vec<Note>, String> {
    let index_path = Path::new(&notebook_path).join(".bunko").join("index.json");
    if index_path.exists() {
        let content = std::fs::read_to_string(&index_path)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content)
            .map_err(|e| e.to_string())
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
async fn get_note_content(notebook_path: String, relative_path: String) -> Result<String, String> {
    let full_path = Path::new(&notebook_path).join(&relative_path);
    std::fs::read_to_string(full_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_note(notebook_path: String, note_id: String, content: String) -> Result<(), String> {
    notebook::update_note(Path::new(&notebook_path), &note_id, &content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_vaults() -> Result<Vec<Vault>, String> {
    vault::list_vaults()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_vault(name: String, path: String) -> Result<(), String> {
    vault::create_vault(&name, &path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_vault(path: String) -> Result<(), String> {
    vault::validate_vault(&path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_vault(path: String) -> Result<(), String> {
    vault::remove_vault(&path)
        .map_err(|e| e.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            init_notebook,
            create_note,
            list_notes,
            get_note_content,
            update_note,
            list_vaults,
            create_vault,
            validate_vault,
            remove_vault
        ])
        .setup(|app| {
            match app.cli().matches() {
                Ok(matches) => {
                    // Parse arguments
                    let path = matches.args.get("path")
                        .and_then(|arg| arg.value.as_str())
                        .unwrap_or(".");
                    
                    let use_tui = matches.args.get("tui")
                        .and_then(|arg| arg.value.as_bool())
                        .unwrap_or(false);
                    
                    // Convert relative path to absolute
                    let notebook_path = if std::path::Path::new(path).is_absolute() {
                        path.to_string()
                    } else {
                        std::env::current_dir()
                            .unwrap()
                            .join(path)
                            .to_string_lossy()
                            .to_string()
                    };
                    
                    if use_tui {
                        // Launch terminal UI
                        println!("Launching TUI mode...");
                        if let Err(e) = tui::run_tui(&notebook_path) {
                            eprintln!("TUI error: {}", e);
                        }
                        std::process::exit(0);
                    } else {
                        // GUI mode - Tauri will continue normal startup
                    }
                }
                Err(_) => {
                    // No CLI args - GUI mode with default behavior
                }
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}