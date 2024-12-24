use bunko_lib::notebook;
use tempfile::tempdir;

#[test]
fn test_notebook_operations() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    
    notebook::init_notebook(path).unwrap();
    assert!(path.join(".bunko").exists());
    
    let note = notebook::add_note(path, "test", "# Test Content").unwrap();
    assert_eq!(note.title, "test");
    assert!(path.join("test.md").exists());
}
