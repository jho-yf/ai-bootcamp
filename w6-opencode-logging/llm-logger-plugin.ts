/// <reference types="node" />
import fs from "fs"
import path from "path"

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const LOG_DIR = path.resolve(process.cwd(), "logs")

// ---------------------------------------------------------------------------
// Inline types (avoids external @opencode-ai/plugin dependency in IDE)
// ---------------------------------------------------------------------------

interface HookInput {
  sessionID: string
  agent: string
  model: { providerID: string; id: string; [k: string]: unknown }
  provider: Record<string, unknown>
  message: Record<string, unknown>
}

interface TurnState {
  turn: number
  sessionID: string
  filePath: string
  ts: string
  userText: string
  inputToolNames: Set<string>
  outputTexts: string[]
  outputToolNames: string[]
  finishReason: string
  isComplete: boolean
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

const turnCounters = new Map<string, number>()
const activeTurns = new Map<string, TurnState>()

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function safeSnapshot(obj: unknown): unknown {
  try {
    return JSON.parse(JSON.stringify(obj))
  } catch {
    return String(obj)
  }
}

async function appendLine(filePath: string, data: Record<string, unknown>) {
  try {
    await fs.promises.appendFile(filePath, JSON.stringify(data) + "\n", "utf-8")
  } catch (err) {
    console.error("[llm-logger] write error:", err)
  }
}

// ---------------------------------------------------------------------------
// Filename summary generation (English only)
// ---------------------------------------------------------------------------

const STOP_WORDS = new Set([
  "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
  "have", "has", "had", "do", "does", "did", "will", "would", "could",
  "should", "may", "might", "can", "shall", "to", "of", "in", "for",
  "on", "with", "at", "by", "from", "as", "into", "about", "it", "its",
  "this", "that", "these", "those", "i", "me", "my", "we", "our", "you",
  "your", "he", "him", "his", "she", "her", "they", "them", "their",
  "and", "or", "but", "not", "no", "if", "then", "than", "so", "just",
])

/** Extract English-only keywords from text */
function extractEnglishKeywords(text: string, maxWords: number): string[] {
  if (!text) return []
  const words = text
    .replace(/[^a-zA-Z0-9\s]/g, " ")
    .split(/\s+/)
    .filter((w) => w.length > 2 && !STOP_WORDS.has(w.toLowerCase()))
  return words.slice(0, maxWords).map((w) => w.toLowerCase())
}

function sanitizeForFilename(s: string): string {
  return s
    .replace(/[^a-zA-Z0-9\-]/g, "")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 80)
}

function generateSummary(state: TurnState): string {
  const parts: string[] = []

  // --- Input side ---
  if (state.turn === 1 && state.userText) {
    const kw = extractEnglishKeywords(state.userText, 3)
    if (kw.length > 0) {
      parts.push(...kw)
    } else {
      parts.push("user-request")
    }
  } else if (state.inputToolNames.size > 0) {
    const names = [...state.inputToolNames].slice(0, 2)
    parts.push(...names, "results")
  }

  // --- Output side ---
  if (state.outputToolNames.length > 0) {
    const unique = [...new Set(state.outputToolNames)].slice(0, 2)
    parts.push("calls", ...unique)
  } else if (state.finishReason === "stop" && state.outputTexts.length > 0) {
    const combined = state.outputTexts.join(" ")
    const kw = extractEnglishKeywords(combined, 2)
    if (parts.length > 0) {
      parts.push("responds", ...kw)
    } else if (kw.length > 0) {
      parts.push(...kw)
    } else {
      parts.push("final-response")
    }
  }

  if (parts.length === 0) {
    parts.push("turn", String(state.turn))
  }

  return sanitizeForFilename(parts.slice(0, 5).join("-"))
}

// ---------------------------------------------------------------------------
// Extract info from input messages for summary
// ---------------------------------------------------------------------------

interface MessageLike {
  info?: { role?: string; sessionID?: string; [k: string]: unknown }
  parts?: Array<Record<string, unknown>>
}

function analyzeInputMessages(messages: MessageLike[]): {
  userText: string
  inputToolNames: Set<string>
} {
  let userText = ""
  const inputToolNames = new Set<string>()

  for (const msg of messages) {
    const role = msg.info?.role
    const parts = msg.parts || []

    if (role === "user" && !userText) {
      for (const part of parts) {
        if (part.type === "text" && typeof part.text === "string" && part.text.trim()) {
          userText = part.text.trim()
          break
        }
      }
    }

    if (role === "assistant") {
      for (const part of parts) {
        const toolName =
          part.toolName ?? part.name ?? (part.tool as Record<string, unknown> | undefined)?.name
        if (typeof toolName === "string") {
          inputToolNames.add(toolName)
        }
      }
    }
  }

  return { userText, inputToolNames }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

const plugin = {
  id: "llm-logger",

  async server(_input: unknown): Promise<Record<string, unknown>> {
    return {
      // 1. Full message array sent to LLM (fires once per turn)
      async "experimental.chat.messages.transform"(
        _input: Record<string, unknown>,
        output: { messages: MessageLike[] },
      ) {
        const messages = output.messages
        if (!messages?.length) return

        const sessionID = messages[0]?.info?.sessionID
        if (!sessionID) return

        const turn = (turnCounters.get(sessionID) ?? 0) + 1
        turnCounters.set(sessionID, turn)

        const sessionDir = path.join(LOG_DIR, sessionID)
        await fs.promises.mkdir(sessionDir, { recursive: true })

        const now = new Date()
        const ts = now.toISOString().replace(/[-:]/g, "").replace("T", "_").split(".")[0]
        const pendingPath = path.join(sessionDir, `${ts}_pending.jsonl`)

        const { userText, inputToolNames } = analyzeInputMessages(messages)

        const state: TurnState = {
          turn,
          sessionID,
          filePath: pendingPath,
          ts,
          userText,
          inputToolNames,
          outputTexts: [],
          outputToolNames: [],
          finishReason: "",
          isComplete: false,
        }
        activeTurns.set(sessionID, state)

        await appendLine(pendingPath, { type: "meta", turn, ts: now.toISOString(), sessionID })
        await appendLine(pendingPath, {
          type: "request",
          messageCount: messages.length,
          messages: safeSnapshot(messages),
        })
      },

      // 2. System prompts
      async "experimental.chat.system.transform"(
        input: { sessionID?: string; model: unknown },
        output: { system: string[] },
      ) {
        const sessionID = input.sessionID
        if (!sessionID) return
        const state = activeTurns.get(sessionID)
        if (!state) return

        await appendLine(state.filePath, {
          type: "system",
          system: safeSnapshot(output.system),
          model: safeSnapshot(input.model),
        })
      },

      // 3. LLM parameters
      async "chat.params"(input: HookInput, output: Record<string, unknown>) {
        const sessionID = input.sessionID
        if (!sessionID) return
        const state = activeTurns.get(sessionID)
        if (!state) return

        await appendLine(state.filePath, {
          type: "params",
          agent: input.agent,
          model: { providerID: input.model.providerID, modelID: input.model.id },
          temperature: output.temperature,
          topP: output.topP,
          topK: output.topK,
          maxOutputTokens: output.maxOutputTokens,
        })
      },

      // 4. Completed text output (fires per text part when streaming ends)
      async "experimental.text.complete"(
        input: { sessionID: string; messageID: string; partID: string },
        output: { text: string },
      ) {
        const sessionID = input.sessionID
        if (!sessionID) return
        const state = activeTurns.get(sessionID)
        if (!state) return

        state.outputTexts.push(output.text)

        await appendLine(state.filePath, {
          type: "text_output",
          messageID: input.messageID,
          partID: input.partID,
          text: output.text,
        })
      },

      // 5. Tool execution results
      async "tool.execute.after"(
        input: { tool: string; sessionID: string; callID: string; args: unknown },
        output: { title: string; output: string; metadata: unknown },
      ) {
        const sessionID = input.sessionID
        if (!sessionID) return
        const state = activeTurns.get(sessionID)
        if (!state) return

        state.outputToolNames.push(input.tool)

        await appendLine(state.filePath, {
          type: "tool_call",
          tool: input.tool,
          callID: input.callID,
          args: safeSnapshot(input.args),
          result: {
            title: output.title,
            output: output.output,
            metadata: safeSnapshot(output.metadata),
          },
        })
      },

      // 6. Bus events: capture final assistant message metadata + rename file
      async "event"(input: { event: Record<string, unknown> }) {
        const event = input.event
        if (!event || typeof event !== "object") return

        if (event.type !== "message.updated") return
        const props = event.properties as Record<string, unknown> | undefined
        if (!props) return

        const sessionID = props.sessionID as string | undefined
        const info = props.info as Record<string, unknown> | undefined
        if (!sessionID || !info) return
        if (info.role !== "assistant" || !info.finish) return

        const state = activeTurns.get(sessionID)
        if (!state || state.isComplete) return

        state.isComplete = true
        state.finishReason = info.finish as string

        await appendLine(state.filePath, {
          type: "response",
          finish: info.finish,
          model: safeSnapshot(info.model),
          tokens: safeSnapshot(info.tokens),
          cost: info.cost,
          error: safeSnapshot(info.error),
        })

        const summary = generateSummary(state)
        const dir = path.dirname(state.filePath)
        const finalName = `${state.ts}_${summary}.jsonl`
        const finalPath = path.join(dir, finalName)

        try {
          await fs.promises.rename(state.filePath, finalPath)
        } catch {
          // Rename failed (e.g. name collision) — data is safe in pending file
        }

        activeTurns.delete(sessionID)
      },
    }
  },
}

export default plugin
