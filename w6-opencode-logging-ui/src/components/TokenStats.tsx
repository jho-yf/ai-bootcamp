import type { ResponseRecord } from '../types'

interface Props {
  response: ResponseRecord
}

export default function TokenStats({ response }: Props) {
  const { tokens } = response
  return (
    <div
      style={{
        display: 'flex',
        gap: 'var(--space-md)',
        padding: 'var(--space-sm) var(--space-md)',
        background: 'var(--md-fog)',
        borderRadius: 'var(--radius-micro)',
        fontSize: 'var(--font-small)',
        fontFamily: 'var(--font-family-primary)',
        color: 'var(--md-slate)',
        flexWrap: 'wrap',
      }}
    >
      <span>
        input: <strong style={{ color: 'var(--md-ink)' }}>{tokens.input}</strong>
      </span>
      <span>
        output: <strong style={{ color: 'var(--md-ink)' }}>{tokens.output}</strong>
      </span>
      {tokens.reasoning > 0 && (
        <span>
          reasoning: <strong style={{ color: 'var(--md-ink)' }}>{tokens.reasoning}</strong>
        </span>
      )}
      {tokens.cache.read > 0 && (
        <span>
          cache read: <strong style={{ color: 'var(--md-ink)' }}>{tokens.cache.read}</strong>
        </span>
      )}
      {tokens.cache.write > 0 && (
        <span>
          cache write: <strong style={{ color: 'var(--md-ink)' }}>{tokens.cache.write}</strong>
        </span>
      )}
      <span>
        total: <strong style={{ color: 'var(--md-ink)' }}>{tokens.total}</strong>
      </span>
    </div>
  )
}
