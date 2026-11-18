export function socketUrl(path) {
  const proto = window.location.protocol === 'https:' ? 'wss' : 'ws'
  return `${proto}://${window.location.host}${path}`
}

export function openSocket(path, { reconnectMs = 1000, maxMs = 10000 } = {}) {
  const listeners = new Set()
  let ws = null
  let closed = false
  let delay = reconnectMs
  let timer = null

  function connect() {
    ws = new WebSocket(socketUrl(path))
    ws.onopen = () => {
      delay = reconnectMs
    }
    ws.onmessage = (event) => {
      let message
      try {
        message = JSON.parse(event.data)
      } catch {
        return
      }
      listeners.forEach((cb) => cb(message))
    }
    ws.onclose = () => {
      if (closed) return
      timer = setTimeout(connect, delay)
      delay = Math.min(delay * 2, maxMs)
    }
    ws.onerror = () => {
      try {
        ws.close()
      } catch {
        /* já fechado */
      }
    }
  }

  connect()

  return {
    subscribe(cb) {
      listeners.add(cb)
      return () => listeners.delete(cb)
    },
    close() {
      closed = true
      clearTimeout(timer)
      if (ws) ws.close()
    },
  }
}
