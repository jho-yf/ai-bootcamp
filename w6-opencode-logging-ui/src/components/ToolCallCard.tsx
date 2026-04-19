import { useState } from 'react'
import type { ToolCallOutput } from '../types'
import MarkdownRenderer from './MarkdownRenderer'

interface Props {
  toolCall: ToolCallOutput
}

export default function ToolCallCard({ toolCall }: Props) {
  const [expanded, setExpanded] = useState(false)

  const argsPreview = Object.entries(toolCall.args)
    .map(([k, v]) => `${k}: ${String(v)}`)
    .join(', ')

  return (
    <div
      className="card"
      style={{ margin: 'var(--space-sm) 0' }}
    >
      <div
        className="card-header"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--space-sm)',
          cursor: 'pointer',
          userSelect: 'none',
        }}
        onClick={() => setExpanded(!expanded)}
      >
        <span
          style={{
            fontSize: 'var(--font-tiny)',
            transition: 'transform var(--transition-quick)',
            display: 'inline-block',
            transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)',
          }}
        >
          ▶
        </span>
        <span className="badge badge-info">{toolCall.tool}</span>
        <span
          className="text-secondary truncate"
          style={{ fontSize: 'var(--font-small)', flex: 1 }}
        >
          {argsPreview}
        </span>
      </div>
      {expanded && (
        <div className="card-body" style={{ maxHeight: '400px', overflow: 'auto' }}>
          {toolCall.result?.output ? (
            <MarkdownRenderer content={`\`\`\`\n${toolCall.result.output}\n\`\`\``} />
          ) : (
            <span className="text-muted" style={{ fontSize: 'var(--font-small)' }}>
              No output
            </span>
          )}
          {toolCall.result?.metadata?.truncated && (
            <div
              className="text-warning"
              style={{ fontSize: 'var(--font-tiny)', marginTop: 'var(--space-xs)' }}
            >
              Output was truncated
            </div>
          )}
        </div>
      )}
    </div>
  )
}
