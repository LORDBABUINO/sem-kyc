import { beforeEach, describe, expect, test } from 'vitest'
import { api, request } from './api.js'
import { storage, SESSION_KEY, ADMIN_KEY } from './storage.js'
import { mockFetch, clearStorage } from '../test/mocks.js'

beforeEach(() => clearStorage())

describe('request', () => {
  test('throws the server error text on 4xx', async () => {
    mockFetch([['POST', '/api/join', { __status: 400, body: { error: 'nome repetido' } }]])
    await expect(request('POST', '/api/join', { name: 'x' })).rejects.toThrow('nome repetido')
  })

  test('falls back to the status when the body has no error', async () => {
    mockFetch([['GET', '/api/report', { __status: 403, body: null }]])
    await expect(request('GET', '/api/report')).rejects.toThrow('Erro 403')
  })

  test('sends JSON bodies', async () => {
    const fetchMock = mockFetch([['POST', '/api/join', { session: 's1', me: {} }]])
    const out = await api.join('Padaria da Lu')
    expect(out.session).toBe('s1')
    const call = fetchMock.calls[0]
    expect(call.body).toEqual({ name: 'Padaria da Lu' })
    expect(call.headers['Content-Type']).toBe('application/json')
  })
})

describe('player calls', () => {
  test('adds X-Session from storage', async () => {
    storage.set(SESSION_KEY, 'tok-1')
    const fetchMock = mockFetch([['GET', '/api/me', { player: {} }]])
    await api.me()
    expect(fetchMock.calls[0].headers['X-Session']).toBe('tok-1')
  })

  test('payInvoice sends only the given fields', async () => {
    storage.set(SESSION_KEY, 'tok-1')
    const fetchMock = mockFetch([['POST', /^\/api\/invoices\/([^/]+)\/pay$/, { result: 'settled', txid: 'ff' }]])
    await api.payInvoice('ABC123')
    expect(fetchMock.calls[0].body).toEqual({})
    await api.payInvoice('ABC123', { inputs: ['a:0'], payjoin: true })
    expect(fetchMock.calls[1].body).toEqual({ inputs: ['a:0'], payjoin: true })
    expect(fetchMock.calls[1].path).toBe('/api/invoices/ABC123/pay')
  })

  test('answerPayjoin sends null to decline', async () => {
    const fetchMock = mockFetch([['POST', '/api/invoices/ABC123/payjoin', { txid: 'ff' }]])
    await api.answerPayjoin('ABC123', null)
    expect(fetchMock.calls[0].body).toEqual({ contribute: null })
  })

  test('payAuto omits inputs when automatic', async () => {
    const fetchMock = mockFetch([['POST', '/api/autos/12/pay', { txid: 'ff' }]])
    await api.payAuto(12)
    expect(fetchMock.calls[0].body).toEqual({})
    await api.payAuto(12, ['a:0'])
    expect(fetchMock.calls[1].body).toEqual({ inputs: ['a:0'] })
  })
})

describe('admin calls', () => {
  test('adds X-Admin from storage', async () => {
    storage.set(ADMIN_KEY, 'adm')
    const fetchMock = mockFetch([
      ['GET', '/api/admin/room', { players: [] }],
      ['POST', '/api/admin/autos/3/forgive', {}],
      ['POST', '/api/admin/config', { negotiation_secs: 200 }],
    ])
    await api.admin.room()
    await api.admin.forgive(3)
    await api.admin.config({ negotiation_secs: 200 })
    expect(fetchMock.calls.every((c) => c.headers['X-Admin'] === 'adm')).toBe(true)
    expect(fetchMock.calls[2].body).toEqual({ negotiation_secs: 200 })
  })
})
