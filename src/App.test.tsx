import { useState } from "react";
import { Box, Text, Button } from '@mantine/core';
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [message, setMessage] = useState("App loaded successfully!");
  const [notebookPath, setNotebookPath] = useState("");

  const testBackend = async () => {
    try {
      const path = await invoke<string>("get_notebook_path");
      setNotebookPath(path);
      setMessage(`Got notebook path: ${path}`);
    } catch (error) {
      setMessage(`Error: ${error}`);
    }
  };

  return (
    <Box p="xl">
      <Text size="xl" mb="md">Bunko Debug Test</Text>
      <Text mb="md">{message}</Text>
      <Text mb="md">Notebook Path: {notebookPath || "Not set"}</Text>
      <Button onClick={testBackend}>Test Backend Connection</Button>
    </Box>
  );
}

export default App;