import { useState } from 'react'
import type { Turn } from '../types'
import { extractUserText } from '../parser'
import MessageBubble from './MessageBubble'
import SystemPromptSection from './SystemPromptSection'
import TokenStats from './TokenStats'

interface Props {
  turn: Turn
}

export default function TurnCard({ turn }: Props) {
  const [collapsed, setCollapsed] = useState(false)

  const userText = extractUserText(turn.request)
  const agentLabel =
    turn.agents.length > 0
      ? turn.agents.map((a) => `${a.agent}@${a.model.modelID}`).join(', ')
      : ''
  const modelName =
    turn.agents.length > 0 ? turn.agents[0]!.model.modelID : ''

  const textOutputs = turn.outputs.filter((o) => o.kind === 'text')
  const assistantText = textOutputs.map((o) => (o as { kind: 'text'; text: string }).text).join('\n')

  return (
    <div className="card animate-slide-down" style={{ margin: 'var(--space-md) 0' }}>
      <div
        className="card-header"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--space-sm)',
          cursor: 'pointer',
          userSelect: 'none',
        }}
        onClick={() => setCollapsed(!collapsed)}
      >
        <span
          style={{
            display: 'inline-block',
            transition: 'transform var(--transition-quick)',
            transform: collapsed ? 'rotate(0deg)' : 'rotate(90deg)',
          }}
        >
          ▶
        </span>
        <strong>Turn {turn.turnNumber}</strong>
        <span className="text-muted" style={{ fontSize: 'var(--font-tiny)' }}>
          {new Date(turn.startTime).toLocaleTimeString()}
        </span>
        {agentLabel && (
          <span className="badge badge-info" style={{ fontSize: 'var(--font-tiny)' }}>
            {agentLabel}
          </span>
        )}
      </div>

      {!collapsed && (
        <div className="card-body">
          <SystemPromptSection prompts={turn.systemPrompts} />

          <MessageBubble
            role="user"
            model={modelName}
            content={userText}
            outputs={[]}
          />

          <MessageBubble
            role="assistant"
            model={modelName}
            content={assistantText}
            outputs={turn.outputs.filter((o) => o.kind === 'tool_call')}
            finish={turn.response?.finish}
          />

          {turn.response && <TokenStats response={turn.response} />}
        </div>
      )}
    </div>
  )
}
