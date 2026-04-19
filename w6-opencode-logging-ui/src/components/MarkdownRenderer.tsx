import Markdown from 'react-markdown'
import rehypeHighlight from 'rehype-highlight'

interface Props {
  content: string
}

export default function MarkdownRenderer({ content }: Props) {
  return (
    <div className="markdown-content">
      <Markdown rehypePlugins={[rehypeHighlight]}>{content}</Markdown>
    </div>
  )
}
