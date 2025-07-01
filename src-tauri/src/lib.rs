use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,  // UUID
    pub title: String,
    pub relative_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notebook {
    path: PathBuf,
    notes: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vault {
    pub name: String,
    pub path: String,
    pub last_opened: Option<DateTime<Utc>>,
    pub note_count: Option<usize>,
}

pub mod notebook {
    use super::*;
    use std::fs;
    
    pub fn init_notebook(path: &Path) -> std::io::Result<()> {
        let bunko_dir = path.join(".bunko");
        if !bunko_dir.exists() {
            fs::create_dir_all(&bunko_dir)?;
            fs::write(
                bunko_dir.join("index.json"),
                serde_json::to_string_pretty(&Vec::<Note>::new())?
            )?;
        }
        Ok(())
    }

    pub fn add_note(notebook_path: &Path, title: &str, content: &str) -> std::io::Result<Note> {
        let note_id = uuid::Uuid::new_v4().to_string();
        let relative_path = PathBuf::from(format!("{}.md", title));
        let abs_path = notebook_path.join(&relative_path);
        
        fs::write(&abs_path, content)?;
        
        let note = Note {
            id: note_id,
            title: title.to_string(),
            relative_path,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        update_index(notebook_path, |mut notes| {
            notes.push(note.clone());
            notes
        })?;
        
        Ok(note)
    }

    pub fn update_note(notebook_path: &Path, note_id: &str, content: &str) -> std::io::Result<()> {
        let index_path = notebook_path.join(".bunko").join("index.json");
        let notes: Vec<Note> = if index_path.exists() {
            serde_json::from_str(&fs::read_to_string(&index_path)?)?
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Index not found"));
        };
        
        let note = notes.iter()
            .find(|n| n.id == note_id)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Note not found"))?;
        
        let abs_path = notebook_path.join(&note.relative_path);
        fs::write(&abs_path, content)?;
        
        update_index(notebook_path, |mut notes| {
            if let Some(note) = notes.iter_mut().find(|n| n.id == note_id) {
                note.updated_at = Utc::now();
            }
            notes
        })?;
        
        Ok(())
    }

    fn update_index<F>(notebook_path: &Path, update_fn: F) -> std::io::Result<()> 
    where F: FnOnce(Vec<Note>) -> Vec<Note> {
        let index_path = notebook_path.join(".bunko").join("index.json");
        let notes: Vec<Note> = if index_path.exists() {
            serde_json::from_str(&fs::read_to_string(&index_path)?)?
        } else {
            Vec::new()
        };
        
        let updated_notes = update_fn(notes);
        fs::write(index_path, serde_json::to_string_pretty(&updated_notes)?)?;
        Ok(())
    }
}

pub mod vault {
    use super::*;
    use std::fs;
    use dirs;
    
    fn get_vaults_config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bunko");
        
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        
        config_dir.join("vaults.json")
    }
    
    pub fn list_vaults() -> std::io::Result<Vec<Vault>> {
        let config_path = get_vaults_config_path();
        
        if !config_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&config_path)?;
        let vaults: Vec<Vault> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        // Filter out vaults that no longer exist and update note counts
        let mut existing_vaults = Vec::new();
        for mut vault in vaults {
            let vault_path = Path::new(&vault.path);
            if vault_path.exists() && vault_path.join(".bunko").exists() {
                // Update note count
                vault.note_count = get_note_count(&vault_path).ok();
                existing_vaults.push(vault);
            }
        }
        
        // Update the config file with existing vaults only
        save_vaults(&existing_vaults)?;
        
        Ok(existing_vaults)
    }
    
    pub fn create_vault(name: &str, path: &str) -> std::io::Result<()> {
        let vault_path = Path::new(path);
        
        // Initialize the notebook at this path
        crate::notebook::init_notebook(vault_path)?;
        
        // Add to vaults list
        let vault = Vault {
            name: name.to_string(),
            path: path.to_string(),
            last_opened: Some(Utc::now()),
            note_count: Some(0),
        };
        
        add_vault_to_config(vault)?;
        
        Ok(())
    }
    
    pub fn validate_vault(path: &str) -> std::io::Result<()> {
        let vault_path = Path::new(path);
        let bunko_dir = vault_path.join(".bunko");
        
        if !vault_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                "Vault directory does not exist"
            ));
        }
        
        if !bunko_dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                "Not a valid Bunko vault (missing .bunko directory)"
            ));
        }
        
        // Add to vaults list if not already there
        let vault_name = vault_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Vault")
            .to_string();
            
        let vault = Vault {
            name: vault_name,
            path: path.to_string(),
            last_opened: Some(Utc::now()),
            note_count: get_note_count(vault_path).ok(),
        };
        
        add_vault_to_config(vault)?;
        
        Ok(())
    }
    
    pub fn remove_vault(path: &str) -> std::io::Result<()> {
        let mut vaults = list_vaults()?;
        vaults.retain(|v| v.path != path);
        save_vaults(&vaults)?;
        Ok(())
    }
    
    fn add_vault_to_config(new_vault: Vault) -> std::io::Result<()> {
        let mut vaults = list_vaults()?;
        
        // Remove existing entry with same path
        vaults.retain(|v| v.path != new_vault.path);
        
        // Add new entry
        vaults.push(new_vault);
        
        // Sort by last opened (most recent first)
        vaults.sort_by(|a, b| {
            match (a.last_opened, b.last_opened) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        
        save_vaults(&vaults)
    }
    
    fn save_vaults(vaults: &[Vault]) -> std::io::Result<()> {
        let config_path = get_vaults_config_path();
        let content = serde_json::to_string_pretty(vaults)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(config_path, content)
    }
    
    fn get_note_count(vault_path: &Path) -> std::io::Result<usize> {
        let index_path = vault_path.join(".bunko").join("index.json");
        if !index_path.exists() {
            return Ok(0);
        }
        
        let content = fs::read_to_string(&index_path)?;
        let notes: Vec<Note> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        Ok(notes.len())
    }
}
