#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_init_notebook() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        
        notebook::init_notebook(path).unwrap();
        
        assert!(path.join(".bunko").exists());
        assert!(path.join(".bunko/index.json").exists());
    }
    
    #[test]
    fn test_add_note() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        notebook::init_notebook(path).unwrap();
        
        let note = notebook::add_note(path, "test", "content").unwrap();
        
        assert_eq!(note.title, "test");
        assert!(path.join("test.md").exists());
    }
}
