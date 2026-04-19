import type { TurnOutput } from '../types'
import MarkdownRenderer from './MarkdownRenderer'
import ToolCallCard from './ToolCallCard'

interface Props {
  role: 'user' | 'assistant'
  model?: string
  content: string
  outputs: TurnOutput[]
  finish?: string
}

export default function MessageBubble({ role, model, content, outputs, finish }: Props) {
  const badgeClass = role === 'user' ? 'badge badge-user' : 'badge badge-assistant'
  const bgColor = role === 'user' ? 'var(--md-soft-blue)' : 'rgba(34, 197, 94, 0.05)'

  return (
    <div
      style={{
        background: bgColor,
        border: 'var(--border-strong)',
        borderRadius: 'var(--radius-micro)',
        padding: 'var(--space-md)',
        margin: 'var(--space-sm) 0',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--space-sm)',
          marginBottom: content || outputs.length > 0 ? 'var(--space-sm)' : 0,
          fontSize: 'var(--font-small)',
        }}
      >
        <span className={badgeClass}>{role}</span>
        {model && <span className="text-muted">{model}</span>}
        {finish && (
          <span className={`badge ${finish === 'stop' ? 'badge-success' : 'badge-warning'}`}>
            {finish}
          </span>
        )}
      </div>

      {content && (
        <div style={{ marginBottom: outputs.length > 0 ? 'var(--space-sm)' : 0 }}>
          <MarkdownRenderer content={content} />
        </div>
      )}

      {outputs.map((out, i) =>
        out.kind === 'text' ? (
          <div key={i} style={{ margin: 'var(--space-xs) 0' }}>
            <MarkdownRenderer content={out.text} />
          </div>
        ) : (
          <ToolCallCard key={out.callID} toolCall={out} />
        ),
      )}
    </div>
  )
}
