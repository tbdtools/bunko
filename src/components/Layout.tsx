import { useState } from "react";
import { Editor } from "./Editor";
import { Preview } from "./Preview";
import { Sidebar } from "./Sidebar";

export const Layout = () => {
  const [content, setContent] = useState("# Welcome to Bunko");

  return (
    <div style={{ height: "100vh", display: "flex" }}>
      <Sidebar className="w-64" />
      <div style={{ flex: 1, display: "flex" }}>
        <div style={{ flex: 1 }}>
          <Editor
            defaultValue={content}
            onChange={(value) => value && setContent(value)}
          />
        </div>
      </div>
    </div>
  );
};
