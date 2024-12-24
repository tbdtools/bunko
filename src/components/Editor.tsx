import MonacoEditor from "@monaco-editor/react";
import { useRef, useEffect } from "react";
import type { editor } from "monaco-editor";

interface EditorProps {
  defaultValue: string;
  onChange: (value: string | undefined) => void;
}

export const Editor = ({ defaultValue, onChange }: EditorProps) => {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  return (
    <div
      className="h-screen min-h-screen bg-[#1e1e2e]"
      data-testid="monaco-editor"
    >
      <MonacoEditor
        height="100vh"
        defaultLanguage="markdown"
        defaultValue={defaultValue}
        theme="vs-dark"
        onChange={onChange}
        options={{
          minimap: { enabled: false },
          fontSize: 16,
          wordWrap: "on",
          lineNumbers: "off",
          renderWhitespace: "boundary",
          scrollBeyondLastLine: false,
          automaticLayout: true,
        }}
      />
    </div>
  );
};
