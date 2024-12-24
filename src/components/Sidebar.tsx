import { FileTree } from "./FileTree";

interface SidebarProps {
  className?: string;
}

export const Sidebar = ({ className }: SidebarProps) => {
  const sampleData = [
    {
      name: "Projects",
      type: "folder" as const,
      children: [
        { name: "Project A.md", type: "file" as const },
        { name: "Project B.md", type: "file" as const },
      ],
    },
    {
      name: "Notes",
      type: "folder" as const,
      children: [
        { name: "Note 1.md", type: "file" as const },
        { name: "Note 2.md", type: "file" as const },
      ],
    },
  ];

  const handleSelect = (path: string[]) => {
    console.log("Selected:", path.join("/"));
  };

  return (
    <aside className={className}>
      <div className="p-4">
        <h2 className="font-semibold mb-4">Notes</h2>
        <FileTree data={sampleData} onSelect={handleSelect} />
      </div>
    </aside>
  );
};
