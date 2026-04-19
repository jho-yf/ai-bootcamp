export interface TurnStartRecord {
  type: 'turn_start'
  turn: number
  ts: string
}

export interface RequestMessagePart {
  type: string
  text?: string
  id?: string
  sessionID?: string
  messageID?: string
}

export interface RequestMessageInfo {
  role: 'user' | 'assistant'
  time?: { created?: number }
  agent?: string
  model?: { providerID: string; modelID: string }
  id?: string
  sessionID?: string
}

export interface RequestMessage {
  info: RequestMessageInfo
  parts: RequestMessagePart[]
}

export interface RequestRecord {
  type: 'request'
  messageCount: number
  messages: RequestMessage[]
}

export interface SystemRecord {
  type: 'system'
  system: string[]
  model: {
    id?: string
    providerID?: string
    name?: string
  }
}

export interface ParamsRecord {
  type: 'params'
  agent: string
  model: { providerID: string; modelID: string }
}

export interface TextOutputRecord {
  type: 'text_output'
  messageID: string
  partID: string
  text: string
}

export interface ToolCallRecord {
  type: 'tool_call'
  tool: string
  callID: string
  args: Record<string, unknown>
  result?: {
    title?: string
    output?: string
    metadata?: {
      preview?: string
      truncated?: boolean
      count?: number
      loaded?: unknown[]
    }
  }
}

export interface ResponseRecord {
  type: 'response'
  finish: 'stop' | 'tool-calls'
  model: string
  tokens: {
    total: number
    input: number
    output: number
    reasoning: number
    cache: { write: number; read: number }
  }
  cost: number
  error: string
}

export type JsonlRecord =
  | TurnStartRecord
  | RequestRecord
  | SystemRecord
  | ParamsRecord
  | TextOutputRecord
  | ToolCallRecord
  | ResponseRecord

export interface TextOutput {
  kind: 'text'
  text: string
}

export interface ToolCallOutput {
  kind: 'tool_call'
  tool: string
  callID: string
  args: Record<string, unknown>
  result?: ToolCallRecord['result']
}

export type TurnOutput = TextOutput | ToolCallOutput

export interface Turn {
  turnNumber: number
  startTime: string
  agents: { agent: string; model: { providerID: string; modelID: string } }[]
  systemPrompts: { agent?: string; model?: { id?: string; providerID?: string; name?: string }; text: string }[]
  request: RequestRecord | null
  outputs: TurnOutput[]
  response: ResponseRecord | null
}

export interface ConversationFile {
  fileName: string
  turns: Turn[]
}

export interface SessionData {
  sessionName: string
  files: string[]
}
