import ReactMarkdown from "react-markdown";

interface PreviewProps {
  content: string;
}

export const Preview = ({ content }: PreviewProps) => (
  <div
    className="p-4 prose prose-zinc max-w-none"
    data-testid="markdown-preview"
  >
    <ReactMarkdown>{content}</ReactMarkdown>
  </div>
);
