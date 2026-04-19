import { useState } from 'react'

interface Props {
  prompts: { text: string; model?: { name?: string; id?: string } }[]
}

export default function SystemPromptSection({ prompts }: Props) {
  const [expanded, setExpanded] = useState(false)

  if (prompts.length === 0) return null

  return (
    <div style={{ margin: 'var(--space-sm) 0' }}>
      <button
        className="btn btn-ghost"
        onClick={() => setExpanded(!expanded)}
        style={{ fontSize: 'var(--font-small)', display: 'flex', alignItems: 'center', gap: 'var(--space-xs)' }}
      >
        <span
          style={{
            display: 'inline-block',
            transition: 'transform var(--transition-quick)',
            transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)',
          }}
        >
          ▶
        </span>
        <span className="badge badge-system">system prompt</span>
        <span className="text-muted">({prompts.length})</span>
      </button>
      {expanded && (
        <div
          className="card"
          style={{ marginTop: 'var(--space-xs)', maxHeight: '300px', overflow: 'auto' }}
        >
          <div className="card-body">
            {prompts.map((p, i) => (
              <div key={i} style={{ marginBottom: i < prompts.length - 1 ? 'var(--space-md)' : 0 }}>
                {p.model?.name && (
                  <div style={{ fontSize: 'var(--font-tiny)', color: 'var(--md-slate)', marginBottom: 'var(--space-xs)' }}>
                    model: {p.model.name} ({p.model.id})
                  </div>
                )}
                <div
                  className="markdown-content"
                  style={{
                    fontSize: 'var(--font-small)',
                    maxHeight: '200px',
                    overflow: 'auto',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                  }}
                >
                  {p.text.length > 2000 ? p.text.slice(0, 2000) + '...' : p.text}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
