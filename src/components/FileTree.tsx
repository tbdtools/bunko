import React, { useState } from "react";
import { ChevronRight, ChevronDown, File, Folder } from "lucide-react";

interface TreeNode {
  name: string;
  type: "file" | "folder";
  children?: TreeNode[];
}

interface FileTreeProps {
  data: TreeNode[];
  onSelect: (path: string[]) => void;
}

const TreeItem = ({
  node,
  path = [],
  onSelect,
}: {
  node: TreeNode;
  path?: string[];
  onSelect: (path: string[]) => void;
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const currentPath = [...path, node.name];

  const handleClick = () => {
    if (node.type === "folder") {
      setIsOpen(!isOpen);
    } else {
      onSelect(currentPath);
    }
  };

  return (
    <div className="select-none">
      <div
        className="flex items-center gap-1 px-2 py-1 hover:bg-zinc-100 cursor-pointer"
        onClick={handleClick}
      >
        <span className="w-4">
          {node.type === "folder" &&
            (isOpen ? <ChevronDown size={16} /> : <ChevronRight size={16} />)}
        </span>
        <span className="w-4">
          {node.type === "folder" ? <Folder size={16} /> : <File size={16} />}
        </span>
        <span className="text-sm">{node.name}</span>
      </div>

      {node.type === "folder" && isOpen && node.children && (
        <div className="ml-4">
          {node.children.map((child, index) => (
            <TreeItem
              key={index}
              node={child}
              path={currentPath}
              onSelect={onSelect}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export const FileTree: React.FC<FileTreeProps> = ({ data, onSelect }) => {
  return (
    <div className="overflow-auto">
      {data.map((node, index) => (
        <TreeItem key={index} node={node} onSelect={onSelect} />
      ))}
    </div>
  );
};
