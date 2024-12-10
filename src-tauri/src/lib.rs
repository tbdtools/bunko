use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    id: String,  // UUID
    title: String,
    relative_path: PathBuf,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notebook {
    path: PathBuf,
    notes: Vec<Note>,
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
