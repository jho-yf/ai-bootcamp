import FileOpener from './FileOpener'

interface Props {
  mode: 'idle' | 'single-file' | 'session'
  sessionName: string | null
  sessions: string[]
  currentSession: string | null
  currentConversation: string | null
  conversations: string[]
  onSessionChange: (session: string) => void
  onConversationChange: (conv: string) => void
  onFileSelect: (content: string, fileName: string) => void
  onDirSelect: (files: Map<string, string>, dirName: string) => void
}

export default function Header({
  mode,
  sessionName,
  sessions,
  currentSession,
  currentConversation,
  conversations,
  onSessionChange,
  onConversationChange,
  onFileSelect,
  onDirSelect,
}: Props) {
  return (
    <header
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 'var(--space-md)',
        padding: 'var(--space-sm) var(--space-md)',
        borderBottom: 'var(--border-strong)',
        background: 'var(--md-cloud)',
        flexWrap: 'wrap',
      }}
    >
      <h3 style={{ margin: 0, whiteSpace: 'nowrap' }}>OpenCode LLM Visualizer</h3>

      {mode === 'session' && sessions.length > 0 && (
        <select
          className="btn btn-secondary"
          style={{ fontSize: 'var(--font-small)' }}
          value={currentSession ?? ''}
          onChange={(e) => onSessionChange(e.target.value)}
        >
          {sessions.map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>
      )}

      {mode === 'session' && sessionName && (
        <span className="text-muted" style={{ fontSize: 'var(--font-small)' }}>
          Session: {sessionName}
        </span>
      )}

      {conversations.length > 0 && (
        <select
          className="btn btn-secondary"
          style={{ fontSize: 'var(--font-small)' }}
          value={currentConversation ?? ''}
          onChange={(e) => onConversationChange(e.target.value)}
        >
          {conversations.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
      )}

      <div style={{ flex: 1 }} />
      <FileOpener onFileSelect={onFileSelect} onDirSelect={onDirSelect} />
    </header>
  )
}
