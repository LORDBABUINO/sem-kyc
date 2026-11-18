import { beforeEach, expect, test } from 'vitest'
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react'
import PhoneApp from './PhoneApp.jsx'
import { mockFetch, installFakeWebSocket, FakeWebSocket, clearStorage } from '../../test/mocks.js'
import { makeMe } from '../../test/fixtures.js'
import { storage, SESSION_KEY } from '../../services/storage.js'

beforeEach(() => {
  clearStorage()
  installFakeWebSocket()
})

test('joins with a name, stores the session and asks for the registration fee', async () => {
  const unregistered = makeMe({ player: { ...makeMe().player, registered: false } })
  let registered = false
  const fetchMock = mockFetch([
    ['POST', '/api/join', { session: 'tok-abc', me: unregistered }],
    ['GET', '/api/me', () => (registered ? makeMe() : unregistered)],
    ['POST', '/api/register', () => {
      registered = true
      return { txid: 'aa' }
    }],
  ])
  render(<PhoneApp />)
  fireEvent.change(screen.getByLabelText('nome fantasia'), { target: { value: 'Padaria da Lu' } })
  fireEvent.click(screen.getByText('entrar'))

  await screen.findByText('Você é Padaria da Lu')
  expect(storage.get(SESSION_KEY)).toBe('tok-abc')
  expect(screen.getByText('pagar 10.000 sats')).toBeTruthy()

  fireEvent.click(screen.getByText('pagar 10.000 sats'))
  await screen.findByRole('tab', { name: /Missão/ })
  expect(fetchMock.find('POST', '/api/register').length).toBe(1)
  expect(fetchMock.find('POST', '/api/register')[0].headers['X-Session']).toBe('tok-abc')
  const tabs = screen.getAllByRole('tab').map((t) => t.getAttribute('aria-label'))
  expect(tabs).toEqual(['Missão', 'Carteira', 'Vender', 'Histórico', 'Crachá'])
})

test('shows the join error from the server', async () => {
  mockFetch([['POST', '/api/join', { __status: 400, body: { error: 'nome repetido' } }]])
  render(<PhoneApp />)
  fireEvent.change(screen.getByLabelText('nome fantasia'), { target: { value: 'X' } })
  fireEvent.click(screen.getByText('entrar'))
  await screen.findByText('nome repetido')
})

test('with a stored session, loads /api/me, connects the socket and refetches on every event', async () => {
  storage.set(SESSION_KEY, 'tok-1')
  const fetchMock = mockFetch([['GET', '/api/me', () => makeMe()]])
  render(<PhoneApp />)
  await screen.findByText('Rodada 1')
  expect(FakeWebSocket.last().url).toContain('/ws?session=tok-1')
  const before = fetchMock.find('GET', '/api/me').length
  act(() => FakeWebSocket.last().emit({ event: 'tx', txid: 'ff' }))
  await waitFor(() => expect(fetchMock.find('GET', '/api/me').length).toBe(before + 1))
  act(() => FakeWebSocket.last().emit({ event: 'block', height: 2 }))
  await waitFor(() => expect(fetchMock.find('GET', '/api/me').length).toBe(before + 2))
})

test('an unlock event shows one novidade card leading to the setting', async () => {
  storage.set(SESSION_KEY, 'tok-1')
  mockFetch([['GET', '/api/me', () => makeMe({ player: { ...makeMe().player, unlocked: ['fresh_address'] } })]])
  render(<PhoneApp />)
  await screen.findByText('Rodada 1')
  act(() => FakeWebSocket.last().emit({ event: 'unlock', unlock: 'fresh_address' }))
  const card = await screen.findByTestId('novidade')
  expect(card.textContent).toContain('Seu QR agora muda a cada venda')
  fireEvent.click(screen.getByText('ver modo de recebimento'))
  await screen.findByRole('dialog', { name: 'Configurações' })
  expect(screen.getByLabelText('endereço novo a cada venda').disabled).toBe(false)
  expect(screen.getByLabelText('Payjoin').disabled).toBe(true)
})

test('suspended: red banner, disabled buy and sell, pay at the counter', async () => {
  storage.set(SESSION_KEY, 'tok-1')
  let paid = false
  const auto = { id: 12, player: 'p1', name: 'Padaria da Lu', round: 1, amount: 43200, paid: false, late_rounds: 0 }
  const fetchMock = mockFetch([
    ['GET', '/api/me', () => (paid ? makeMe() : makeMe({ suspended: true, debt: 43200, autos: [auto] }))],
    ['POST', '/api/autos/12/pay', () => {
      paid = true
      return { txid: 'cc' }
    }],
  ])
  render(<PhoneApp />)
  await screen.findByRole('alert')
  expect(screen.getByText(/Auto de Infração DOE nº 12: ICMS devido 43.200 sats/)).toBeTruthy()
  expect(screen.getByText('escanear pra pagar').disabled).toBe(true)
  expect(screen.getAllByText('regularize pra operar').length).toBeGreaterThan(0)

  fireEvent.click(screen.getByRole('tab', { name: /Vender/ }))
  expect(screen.getByText('refeições da colheita').closest('button').disabled).toBe(true)

  fireEvent.click(screen.getByText('pagar no guichê'))
  await screen.findByRole('dialog', { name: 'Guichê do DOE' })
  fireEvent.click(screen.getByText('pagar 43.200 sats'))
  await screen.findByText('Auto quitado.')
  expect(fetchMock.find('POST', '/api/autos/12/pay')[0].body).toEqual({})
  fireEvent.click(screen.getByText('voltar'))
  await waitFor(() => expect(screen.queryByRole('alert')).toBeNull())
})

test('drops a rejected session and goes back to the entry', async () => {
  storage.set(SESSION_KEY, 'stale')
  mockFetch([['GET', '/api/me', { __status: 401, body: { error: 'sessão inválida' } }]])
  render(<PhoneApp />)
  await screen.findByLabelText('nome fantasia')
  expect(storage.get(SESSION_KEY)).toBeNull()
})

test('remembers the current tab', async () => {
  storage.set(SESSION_KEY, 'tok-1')
  mockFetch([['GET', '/api/me', () => makeMe()]])
  render(<PhoneApp />)
  await screen.findByText('Rodada 1')
  fireEvent.click(screen.getByRole('tab', { name: /Crachá/ }))
  expect(storage.get('semkyc.tab')).toBe('badge')
  expect(screen.getByText('Padaria da Lu')).toBeTruthy()
})
