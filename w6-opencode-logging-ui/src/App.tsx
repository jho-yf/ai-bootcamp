import { useState, useCallback } from 'react'
import type { ConversationFile } from './types'
import { parseJsonl } from './parser'
import Header from './components/Header'
import ConversationViewer from './components/ConversationViewer'

type AppMode = 'idle' | 'single-file' | 'session'

interface SessionGroup {
  name: string
  files: Map<string, string>
}

export default function App() {
  const [mode, setMode] = useState<AppMode>('idle')
  const [sessions, setSessions] = useState<SessionGroup[]>([])
  const [currentSessionIdx, setCurrentSessionIdx] = useState<number>(-1)
  const [currentConvName, setCurrentConvName] = useState<string | null>(null)
  const [parsedCache, setParsedCache] = useState<Map<string, ConversationFile>>(new Map())
  const [conversationNames, setConversationNames] = useState<string[]>([])

  const parseAndCache = useCallback(
    (name: string, content: string): ConversationFile => {
      const existing = parsedCache.get(name)
      if (existing) return existing
      const parsed = parseJsonl(content, name)
      setParsedCache((prev) => new Map(prev).set(name, parsed))
      return parsed
    },
    [parsedCache],
  )

  const handleFileSelect = useCallback(
    (content: string, fileName: string) => {
      parseAndCache(fileName, content)
      setMode('single-file')
      setSessions([])
      setCurrentSessionIdx(-1)
      setConversationNames([fileName])
      setCurrentConvName(fileName)
    },
    [parseAndCache],
  )

  const handleDirSelect = useCallback(
    (files: Map<string, string>, _dirName: string) => {
      const grouped = new Map<string, Map<string, string>>()

      for (const [path, content] of files) {
        const parts = path.split('/')
        let sessionName: string
        let fileName: string

        if (parts.length >= 3) {
          sessionName = parts.slice(0, -1).join('/')
          fileName = parts[parts.length - 1]!
        } else if (parts.length === 2) {
          sessionName = parts[0]!
          fileName = parts[1]!
        } else {
          sessionName = '_root'
          fileName = parts[0]!
        }

        if (!grouped.has(sessionName)) {
          grouped.set(sessionName, new Map())
        }
        grouped.get(sessionName)!.set(fileName, content)
      }

      const sessionList = Array.from(grouped.entries())
        .map(([name, files]) => ({ name, files }))
        .sort((a, b) => a.name.localeCompare(b.name))

      for (const session of sessionList) {
        for (const [fname, content] of session.files) {
          parseAndCache(fname, content)
        }
      }

      setMode('session')
      setSessions(sessionList)
      setCurrentSessionIdx(0)

      if (sessionList.length > 0 && sessionList[0]!) {
        const firstSession = sessionList[0]!
        const names = Array.from(firstSession.files.keys()).sort()
        setConversationNames(names)
        setCurrentConvName(names[0] ?? null)
      }
    },
    [parseAndCache],
  )

  const handleSessionChange = useCallback(
    (sessionName: string) => {
      const idx = sessions.findIndex((s) => s.name === sessionName)
      if (idx === -1) return
      setCurrentSessionIdx(idx)
      const session = sessions[idx]!
      const names = Array.from(session.files.keys()).sort()
      setConversationNames(names)
      setCurrentConvName(names[0] ?? null)
    },
    [sessions],
  )

  const handleConversationChange = useCallback((convName: string) => {
    setCurrentConvName(convName)
  }, [])

  const currentConversation = currentConvName ? parsedCache.get(currentConvName) ?? null : null

  const sessionNames = sessions.map((s) => s.name)
  const currentSession =
    currentSessionIdx >= 0 && currentSessionIdx < sessions.length
      ? sessions[currentSessionIdx]!.name
      : null

  return (
    <div style={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <Header
        mode={mode}
        sessionName={currentSession}
        sessions={sessionNames}
        currentSession={currentSession}
        currentConversation={currentConvName}
        conversations={conversationNames}
        onSessionChange={handleSessionChange}
        onConversationChange={handleConversationChange}
        onFileSelect={handleFileSelect}
        onDirSelect={handleDirSelect}
      />

      {mode === 'idle' && (
        <div
          className="drop-zone"
          style={{
            margin: 'var(--space-2xl)',
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <h2 style={{ marginBottom: 'var(--space-md)' }}>OpenCode LLM Visualizer</h2>
          <p className="text-secondary" style={{ marginBottom: 'var(--space-lg)' }}>
            Open a JSONL log file or a session directory to visualize LLM interactions
          </p>
          <div style={{ display: 'flex', gap: 'var(--space-md)' }}>
            <button
              className="btn btn-primary"
              onClick={() => document.getElementById('file-input')?.click()}
            >
              Open JSONL File
            </button>
            <button
              className="btn btn-primary"
              onClick={() => document.getElementById('dir-input')?.click()}
            >
              Open Session Dir
            </button>
          </div>
        </div>
      )}

      {mode !== 'idle' && currentConversation && (
        <ConversationViewer conversation={currentConversation} />
      )}

      {mode !== 'idle' && !currentConversation && (
        <div style={{ textAlign: 'center', padding: 'var(--space-2xl)', color: 'var(--md-slate)' }}>
          Select a conversation to view
        </div>
      )}
    </div>
  )
}
