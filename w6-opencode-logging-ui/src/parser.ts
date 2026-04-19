import type {
  JsonlRecord,
  Turn,
  ConversationFile,
  TurnOutput,
  TurnStartRecord,
  RequestRecord,
  SystemRecord,
  ParamsRecord,
  TextOutputRecord,
  ToolCallRecord,
  ResponseRecord,
} from './types'

function parseLine(line: string): JsonlRecord | null {
  try {
    return JSON.parse(line) as JsonlRecord
  } catch {
    return null
  }
}

export function parseJsonl(content: string, fileName: string): ConversationFile {
  const lines = content.split('\n').filter((l) => l.trim())
  const records: JsonlRecord[] = []
  for (const line of lines) {
    const rec = parseLine(line)
    if (rec) records.push(rec)
  }

  const turns: Turn[] = []
  let currentTurn: Turn | null = null

  function pushTurn() {
    if (currentTurn) turns.push(currentTurn)
  }

  for (const rec of records) {
    switch (rec.type) {
      case 'turn_start': {
        const t = rec as TurnStartRecord
        pushTurn()
        currentTurn = {
          turnNumber: t.turn,
          startTime: t.ts,
          agents: [],
          systemPrompts: [],
          request: null,
          outputs: [],
          response: null,
        }
        break
      }
      case 'request': {
        if (currentTurn) currentTurn.request = rec as RequestRecord
        break
      }
      case 'system': {
        const s = rec as SystemRecord
        if (currentTurn) {
          for (const text of s.system) {
            currentTurn.systemPrompts.push({ model: s.model, text })
          }
        }
        break
      }
      case 'params': {
        const p = rec as ParamsRecord
        if (currentTurn) {
          currentTurn.agents.push({ agent: p.agent, model: p.model })
        }
        break
      }
      case 'text_output': {
        const t = rec as TextOutputRecord
        if (currentTurn) {
          currentTurn.outputs.push({ kind: 'text', text: t.text } as TurnOutput)
        }
        break
      }
      case 'tool_call': {
        const tc = rec as ToolCallRecord
        if (currentTurn) {
          currentTurn.outputs.push({
            kind: 'tool_call',
            tool: tc.tool,
            callID: tc.callID,
            args: tc.args,
            result: tc.result,
          } as TurnOutput)
        }
        break
      }
      case 'response': {
        if (currentTurn) currentTurn.response = rec as ResponseRecord
        break
      }
    }
  }

  pushTurn()

  return { fileName, turns }
}

export function extractUserText(request: RequestRecord | null): string {
  if (!request) return ''
  for (const msg of request.messages) {
    if (msg.info.role === 'user') {
      return msg.parts
        .filter((p) => p.type === 'text' && p.text)
        .map((p) => p.text!)
        .join('\n')
    }
  }
  return ''
}

export function cleanToolOutput(output: string | undefined): string {
  if (!output) return ''
  return output
}
