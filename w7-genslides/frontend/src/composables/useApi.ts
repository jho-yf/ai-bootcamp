const BASE_URL = '/api/ppt'

export function useApi() {
  async function request<T>(path: string, options?: RequestInit): Promise<T> {
    const res = await fetch(`${BASE_URL}${path}`, {
      headers: { 'Content-Type': 'application/json' },
      ...options,
    })

    if (res.status === 204) {
      return undefined as T
    }

    if (!res.ok) {
      const body = await res.json().catch(() => ({ error: { message: res.statusText } }))
      const msg = body?.error?.message || body?.message || `请求失败 (${res.status})`
      throw new Error(msg)
    }

    return res.json()
  }

  function get<T>(path: string): Promise<T> {
    return request<T>(path, { method: 'GET' })
  }

  function post<T>(path: string, body?: unknown): Promise<T> {
    return request<T>(path, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    })
  }

  function put<T>(path: string, body?: unknown): Promise<T> {
    return request<T>(path, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    })
  }

  function del<T>(path: string): Promise<T> {
    return request<T>(path, { method: 'DELETE' })
  }

  return { get, post, put, del }
}
