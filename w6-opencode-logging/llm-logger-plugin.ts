/// <reference types="node" />
import fs from "fs"
import path from "path"

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const LOG_DIR = path.resolve(process.cwd(), "logs")

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

interface SessionState {
  dir: string // e.g. logs/20260417_093500_abc123/
  conversationIndex: number
  filePath: string // current conversation file
  turn: number
  loggedMessages: Set<string>
  conversationDone: boolean // true after finish=stop
}

const sessions = new Map<string, SessionState>()

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

/** Format Date as local time: 20260416_235904 */
function formatLocalTime(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0")
  return (
    `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}` +
    `_${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`
  )
}

// ---------------------------------------------------------------------------
// Filename generation (English only)
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

function extractEnglishKeywords(text: string, maxWords: number): string[] {
  if (!text) return []
  return text
    .replace(/[^a-zA-Z0-9\s]/g, " ")
    .split(/\s+/)
    .filter((w) => w.length > 2 && !STOP_WORDS.has(w.toLowerCase()))
    .slice(0, maxWords)
    .map((w) => w.toLowerCase())
}

function sanitizeForFilename(s: string): string {
  return s
    .replace(/[^a-zA-Z0-9\-]/g, "")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 80)
}

function makeConversationSummary(userText: string): string {
  const kw = extractEnglishKeywords(userText, 4)
  if (kw.length > 0) return sanitizeForFilename(kw.join("-"))
  return "conversation"
}

/** Extract first user text from messages */
function firstUserText(
  messages: Array<{
    info?: { role?: string; [k: string]: unknown }
    parts?: Array<Record<string, unknown>>
  }>,
): string {
  for (const msg of messages) {
    if (msg.info?.role !== "user") continue
    for (const part of msg.parts || []) {
      if (part.type === "text" && typeof part.text === "string" && part.text.trim()) {
        return part.text.trim()
      }
    }
  }
  return ""
}

/** Start a new conversation file within a session */
function startNewConversation(state: SessionState, userText: string) {
  state.conversationIndex++
  state.turn = 0
  state.conversationDone = false
  state.loggedMessages.clear()

  const summary = makeConversationSummary(userText)
  const seq = String(state.conversationIndex).padStart(3, "0")
  state.filePath = path.join(state.dir, `${seq}_${summary}.jsonl`)
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
        output: { messages: Array<Record<string, unknown>> },
      ) {
        const messages = output.messages
        if (!messages?.length) return

        const sessionID = (messages[0] as { info?: { sessionID?: string } })?.info?.sessionID as
          | string
          | undefined
        if (!sessionID) return

        let state = sessions.get(sessionID)

        // Create session folder on first encounter
        if (!state) {
          const now = new Date()
          const ts = formatLocalTime(now)
          const sessionDir = path.join(LOG_DIR, `${ts}_${sessionID}`)
          await fs.promises.mkdir(sessionDir, { recursive: true })
          state = {
            dir: sessionDir,
            conversationIndex: 0,
            filePath: "",
            turn: 0,
            loggedMessages: new Set(),
            conversationDone: false,
          }
          sessions.set(sessionID, state)
        }

        // Start new conversation if: first ever, or previous one finished
        if (state.conversationDone || state.conversationIndex === 0) {
          const userText = firstUserText(
            messages as Array<{
              info?: { role?: string; [k: string]: unknown }
              parts?: Array<Record<string, unknown>>
            }>,
          )
          startNewConversation(state, userText)
        }

        state.turn++

        await appendLine(state.filePath, {
          type: "turn_start",
          turn: state.turn,
          ts: new Date().toISOString(),
        })
        await appendLine(state.filePath, {
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
        const state = sessions.get(sessionID)
        if (!state) return

        await appendLine(state.filePath, {
          type: "system",
          system: safeSnapshot(output.system),
          model: safeSnapshot(input.model),
        })
      },

      // 3. LLM parameters
      async "chat.params"(
        input: {
          sessionID: string
          agent: string
          model: { providerID: string; id: string; [k: string]: unknown }
          [k: string]: unknown
        },
        output: Record<string, unknown>,
      ) {
        const sessionID = input.sessionID
        if (!sessionID) return
        const state = sessions.get(sessionID)
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
        const state = sessions.get(sessionID)
        if (!state) return

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
        const state = sessions.get(sessionID)
        if (!state) return

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

      // 6. Bus events: capture assistant message metadata
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

        const state = sessions.get(sessionID)
        if (!state) return

        // Deduplicate: only log once per message ID
        const msgID = info.id as string | undefined
        if (msgID && state.loggedMessages.has(msgID)) return
        if (msgID) state.loggedMessages.add(msgID)

        await appendLine(state.filePath, {
          type: "response",
          finish: info.finish,
          model: safeSnapshot(info.model),
          tokens: safeSnapshot(info.tokens),
          cost: info.cost,
          error: safeSnapshot(info.error),
        })

        // Mark conversation done when LLM finishes (not tool-calls)
        // finish=tool-calls means more turns are coming
        if (info.finish !== "tool-calls") {
          state.conversationDone = true
        }
      },
    }
  },
}

export default plugin
