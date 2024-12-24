import { render, screen, fireEvent } from "@testing-library/react";
import { FileTree } from "../components/FileTree";

const sampleData = [
  {
    name: "Test Folder",
    type: "folder" as const,
    children: [{ name: "Test File.md", type: "file" as const }],
  },
];

describe("FileTree", () => {
  it("renders folder and file nodes", () => {
    const handleSelect = jest.fn();
    render(<FileTree data={sampleData} onSelect={handleSelect} />);

    expect(screen.getByText("Test Folder")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Test Folder"));
    expect(screen.getByText("Test File.md")).toBeInTheDocument();
  });
});
