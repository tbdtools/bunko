import { render } from "@testing-library/react";
import { Editor } from "../components/Editor";

jest.mock("@monaco-editor/react", () => {
  return function MockMonacoEditor() {
    return <div data-testid="monaco-editor">Monaco Editor</div>;
  };
});

describe("Editor", () => {
  it("renders without crashing", () => {
    render(<Editor defaultValue="# Test" onChange={() => {}} />);
  });
});
