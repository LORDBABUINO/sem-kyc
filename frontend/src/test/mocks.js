import { vi } from 'vitest'

// routes: [method, path | RegExp, handler | payload]; handler(body, match, init) may return
// { __status, body } to answer an error.
export function mockFetch(routes) {
  const calls = []
  const fn = vi.fn(async (url, init = {}) => {
    const method = (init.method || 'GET').toUpperCase()
    const path = String(url).replace(/^https?:\/\/[^/]+/, '')
    const body = init.body ? JSON.parse(init.body) : undefined
    calls.push({ method, path, body, headers: init.headers || {} })
    for (const [m, p, handler] of routes) {
      if (m !== method) continue
      const match = typeof p === 'string' ? (p === path ? [path] : null) : path.match(p)
      if (!match) continue
      const out = typeof handler === 'function' ? await handler(body, match, init) : handler
      const status = out && out.__status ? out.__status : 200
      const payload = out && out.__status ? out.body : out
      return {
        ok: status < 400,
        status,
        json: async () => payload,
        text: async () => JSON.stringify(payload),
      }
    }
    return {
      ok: false,
      status: 404,
      json: async () => ({ error: `sem rota ${method} ${path}` }),
      text: async () => '',
    }
  })
  fn.calls = calls
  fn.find = (method, path) => calls.filter((c) => c.method === method && c.path === path)
  global.fetch = fn
  return fn
}

export class FakeWebSocket {
  static instances = []

  constructor(url) {
    this.url = url
    this.readyState = 0
    this.sent = []
    FakeWebSocket.instances.push(this)
  }

  open() {
    this.readyState = 1
    if (this.onopen) this.onopen()
  }

  emit(message) {
    if (this.onmessage) this.onmessage({ data: JSON.stringify(message) })
  }

  fail() {
    if (this.onerror) this.onerror(new Event('error'))
  }

  close() {
    if (this.readyState === 3) return
    this.readyState = 3
    if (this.onclose) this.onclose()
  }

  send(data) {
    this.sent.push(data)
  }

  static last() {
    return FakeWebSocket.instances[FakeWebSocket.instances.length - 1]
  }

  static reset() {
    FakeWebSocket.instances = []
  }
}

export function installFakeWebSocket() {
  FakeWebSocket.reset()
  global.WebSocket = FakeWebSocket
  return FakeWebSocket
}

export function clearStorage() {
  try {
    window.localStorage.clear()
  } catch {
    /* ignore */
  }
}
