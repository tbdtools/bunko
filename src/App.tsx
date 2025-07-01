import { useState, useEffect, useRef, useCallback } from "react";
import { AppShell, ScrollArea, Text, ActionIcon, TextInput, Group, Stack, Card, Title, UnstyledButton, Box, Button } from '@mantine/core';
import { IconPlus, IconSearch, IconDeviceFloppy } from '@tabler/icons-react';
import Editor from '@monaco-editor/react';
import { invoke } from "@tauri-apps/api/core";
import VaultSelector from './components/VaultSelector';
import "./App.css";

interface Note {
  id: string;
  title: string;
  relative_path: string;
  created_at: string;
  updated_at: string;
}

function App() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);
  const [noteContent, setNoteContent] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [currentVaultPath, setCurrentVaultPath] = useState<string | null>(null);
  const [showVaultSelector, setShowVaultSelector] = useState(true);
  const editorRef = useRef<any>(null);

  useEffect(() => {
    // Initialize notebook when vault is selected
    if (currentVaultPath) {
      initNotebook();
    }
  }, [currentVaultPath]);

  // Auto-save functionality
  useEffect(() => {
    if (hasUnsavedChanges && selectedNote) {
      const timer = setTimeout(() => {
        saveNote();
      }, 1000); // Auto-save after 1 second of inactivity

      return () => clearTimeout(timer);
    }
  }, [noteContent, hasUnsavedChanges, selectedNote]);

  const initNotebook = async () => {
    if (!currentVaultPath) return;
    
    try {
      await invoke("init_notebook", { path: currentVaultPath });
      loadNotes();
    } catch (error) {
      console.error("Failed to initialize notebook:", error);
    }
  };

  const loadNotes = async () => {
    if (!currentVaultPath) return;
    
    try {
      const notesList = await invoke<Note[]>("list_notes", { notebookPath: currentVaultPath });
      setNotes(notesList);
    } catch (error) {
      console.error("Failed to load notes:", error);
    }
  };

  const createNewNote = async () => {
    if (!currentVaultPath) return;
    
    const title = `New Note ${new Date().toLocaleString()}`;
    const content = "# " + title + "\n\nStart writing...";
    
    try {
      const newNote = await invoke<Note>("create_note", {
        notebookPath: currentVaultPath,
        title,
        content
      });
      await loadNotes();
      setSelectedNote(newNote);
      setNoteContent(content);
      setHasUnsavedChanges(false);
    } catch (error) {
      console.error("Failed to create note:", error);
    }
  };

  const loadNoteContent = async (note: Note) => {
    if (!currentVaultPath) return;
    
    // Save current note if there are unsaved changes
    if (hasUnsavedChanges && selectedNote) {
      await saveNote();
    }

    try {
      const content = await invoke<string>("get_note_content", {
        notebookPath: currentVaultPath,
        relativePath: note.relative_path
      });
      setNoteContent(content);
      setSelectedNote(note);
      setHasUnsavedChanges(false);
    } catch (error) {
      console.error("Failed to load note content:", error);
    }
  };

  const saveNote = async () => {
    if (!selectedNote || !hasUnsavedChanges || !currentVaultPath) return;

    setIsSaving(true);
    try {
      await invoke("update_note", {
        notebookPath: currentVaultPath,
        noteId: selectedNote.id,
        content: noteContent
      });
      setHasUnsavedChanges(false);
      await loadNotes(); // Reload to get updated timestamps
    } catch (error) {
      console.error("Failed to save note:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleEditorChange = (value: string | undefined) => {
    if (value !== undefined) {
      setNoteContent(value);
      setHasUnsavedChanges(true);
    }
  };

  const handleEditorMount = (editor: any) => {
    editorRef.current = editor;
  };

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // Cmd/Ctrl + S to save
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault();
      if (hasUnsavedChanges) {
        saveNote();
      }
    }
  }, [hasUnsavedChanges, saveNote]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const handleVaultSelected = (vaultPath: string) => {
    setCurrentVaultPath(vaultPath);
    setShowVaultSelector(false);
    setNotes([]);
    setSelectedNote(null);
    setNoteContent("");
    setHasUnsavedChanges(false);
  };

  const filteredNotes = notes.filter(note => 
    note.title.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <>
      <VaultSelector 
        opened={showVaultSelector}
        onVaultSelected={handleVaultSelected}
      />
      
      <AppShell
      navbar={{
        width: 300,
        breakpoint: 'sm',
      }}
      padding={0}
    >
      <AppShell.Navbar p="md">
        <Stack gap="sm">
          <Group justify="space-between">
            <Title order={4}>Notes</Title>
            <ActionIcon 
              variant="filled" 
              size="lg"
              onClick={createNewNote}
              title="Create new note"
            >
              <IconPlus size={20} />
            </ActionIcon>
          </Group>
          
          <TextInput
            placeholder="Search notes..."
            leftSection={<IconSearch size={16} />}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.currentTarget.value)}
          />

          <ScrollArea style={{ height: 'calc(100vh - 160px)' }}>
            <Stack gap="xs">
              {filteredNotes.map((note) => (
                <UnstyledButton
                  key={note.id}
                  onClick={() => loadNoteContent(note)}
                >
                  <Card
                    padding="md"
                    withBorder
                    style={{
                      backgroundColor: selectedNote?.id === note.id ? 'var(--mantine-color-dark-5)' : undefined,
                      cursor: 'pointer'
                    }}
                  >
                    <Text fw={500} size="sm">{note.title}</Text>
                    <Text size="xs" c="dimmed">
                      {new Date(note.updated_at).toLocaleDateString()}
                    </Text>
                  </Card>
                </UnstyledButton>
              ))}
              
              {filteredNotes.length === 0 && (
                <Text c="dimmed" ta="center" mt="xl">
                  {searchQuery ? "No notes found" : "No notes yet. Create your first note!"}
                </Text>
              )}
            </Stack>
          </ScrollArea>
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main>
        <Box style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          {selectedNote ? (
            <>
              <Box p="md" style={{ borderBottom: '1px solid var(--mantine-color-dark-4)' }}>
                <Group justify="space-between">
                  <Title order={3}>{selectedNote.title}</Title>
                  <Group>
                    {hasUnsavedChanges && (
                      <Text size="sm" c="dimmed">Unsaved changes</Text>
                    )}
                    {isSaving && (
                      <Text size="sm" c="dimmed">Saving...</Text>
                    )}
                    <Button 
                      size="sm"
                      leftSection={<IconDeviceFloppy size={16} />}
                      onClick={saveNote}
                      disabled={!hasUnsavedChanges || isSaving}
                    >
                      Save
                    </Button>
                  </Group>
                </Group>
              </Box>
              <Box style={{ flex: 1 }}>
                <Editor
                  height="100%"
                  defaultLanguage="markdown"
                  theme="vs-dark"
                  value={noteContent}
                  onChange={handleEditorChange}
                  onMount={handleEditorMount}
                  options={{
                    minimap: { enabled: false },
                    fontSize: 14,
                    lineNumbers: 'off',
                    wordWrap: 'on',
                    padding: { top: 16, bottom: 16 },
                    scrollBeyondLastLine: false,
                    automaticLayout: true,
                  }}
                />
              </Box>
            </>
          ) : (
            <Box ta="center" mt="xl" p="xl">
              <Text c="dimmed" size="lg">
                Select a note from the sidebar or create a new one
              </Text>
            </Box>
          )}
        </Box>
      </AppShell.Main>
      </AppShell>
    </>
  );
}

export default App;