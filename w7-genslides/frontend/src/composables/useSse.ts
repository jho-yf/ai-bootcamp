import { ref, onUnmounted } from 'vue'

type EventCallback = (data: unknown) => void

export function useSse() {
  const connected = ref(false)
  const error = ref<string | null>(null)

  let abortController: AbortController | null = null
  const listeners = new Map<string, EventCallback[]>()

  async function connect(url: string) {
    abortController = new AbortController()
    connected.value = true

    try {
      const res = await fetch(url, {
        method: 'POST',
        signal: abortController.signal,
      })

      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        const msg = body?.error?.message || `连接失败 (${res.status})`
        error.value = msg
        connected.value = false
        for (const cb of listeners.get('error') ?? []) {
          cb({ error: msg })
        }
        return
      }

      const reader = res.body?.getReader()
      if (!reader) return

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() ?? ''

        let eventType = 'message'
        let eventData = ''

        for (const line of lines) {
          if (line.startsWith('event: ')) {
            eventType = line.slice(7).trim()
          } else if (line.startsWith('data: ')) {
            eventData = line.slice(6)
          } else if (line === '' && eventData) {
            const callbacks = listeners.get(eventType)
            if (callbacks) {
              try {
                const data = JSON.parse(eventData)
                for (const cb of callbacks) cb(data)
              } catch {
                console.error('Failed to parse SSE data:', eventData)
              }
            }
            eventType = 'message'
            eventData = ''
          }
        }
      }
    } catch (e) {
      if ((e as Error).name === 'AbortError') return
      error.value = e instanceof Error ? e.message : '连接失败'
      connected.value = false
      for (const cb of listeners.get('error') ?? []) {
        cb({ error: error.value })
      }
    } finally {
      connected.value = false
    }
  }

  function on(event: string, callback: EventCallback) {
    if (!listeners.has(event)) {
      listeners.set(event, [])
    }
    listeners.get(event)!.push(callback)
  }

  function close() {
    abortController?.abort()
    abortController = null
    connected.value = false
  }

  onUnmounted(() => {
    close()
  })

  return { connected, error, connect, on, close }
}
