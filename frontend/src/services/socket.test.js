import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { openSocket, socketUrl } from './socket.js'
import { installFakeWebSocket, FakeWebSocket } from '../test/mocks.js'

beforeEach(() => {
  installFakeWebSocket()
  vi.useFakeTimers()
})
afterEach(() => vi.useRealTimers())

test('builds ws url from the page location', () => {
  expect(socketUrl('/ws/projector')).toBe(`ws://${window.location.host}/ws/projector`)
})

test('delivers parsed messages to subscribers and ignores garbage', () => {
  const sock = openSocket('/ws?session=t')
  const seen = []
  sock.subscribe((m) => seen.push(m))
  const ws = FakeWebSocket.last()
  ws.open()
  ws.emit({ event: 'phase', phase: { phase: 'lobby' } })
  ws.onmessage({ data: 'not json' })
  expect(seen).toEqual([{ event: 'phase', phase: { phase: 'lobby' } }])
  sock.close()
})

test('reconnects with backoff after an unexpected close', () => {
  const sock = openSocket('/ws/projector', { reconnectMs: 1000, maxMs: 4000 })
  expect(FakeWebSocket.instances.length).toBe(1)
  FakeWebSocket.last().close()
  vi.advanceTimersByTime(999)
  expect(FakeWebSocket.instances.length).toBe(1)
  vi.advanceTimersByTime(1)
  expect(FakeWebSocket.instances.length).toBe(2)
  FakeWebSocket.last().close()
  vi.advanceTimersByTime(2000)
  expect(FakeWebSocket.instances.length).toBe(3)
  sock.close()
  vi.advanceTimersByTime(60000)
  expect(FakeWebSocket.instances.length).toBe(3)
})

test('unsubscribe stops delivery', () => {
  const sock = openSocket('/ws/admin?token=x')
  const cb = vi.fn()
  const off = sock.subscribe(cb)
  off()
  FakeWebSocket.last().emit({ event: 'block', height: 2 })
  expect(cb).not.toHaveBeenCalled()
  sock.close()
})
