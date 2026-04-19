import type { ConversationFile } from '../types'
import TurnCard from './TurnCard'

interface Props {
  conversation: ConversationFile
}

export default function ConversationViewer({ conversation }: Props) {
  if (conversation.turns.length === 0) {
    return (
      <div style={{ textAlign: 'center', padding: 'var(--space-2xl)', color: 'var(--md-slate)' }}>
        No turns found in this conversation.
      </div>
    )
  }

  return (
    <div style={{ padding: 'var(--space-md)', flex: 1, overflowY: 'auto' }}>
      {conversation.turns.map((turn) => (
        <TurnCard key={turn.turnNumber} turn={turn} />
      ))}
    </div>
  )
}
