import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react'
import PresenterApp from './PresenterApp.jsx'
import { mockFetch, installFakeWebSocket, FakeWebSocket, clearStorage } from '../../test/mocks.js'
import { makeState } from '../../test/fixtures.js'
import { storage, ADMIN_KEY } from '../../services/storage.js'

beforeEach(() => {
  clearStorage()
  installFakeWebSocket()
})
afterEach(() => vi.restoreAllMocks())

function makeRoom(overrides = {}) {
  return {
    state: makeState({
      phase: { phase: 'negotiation', round: 2, deadline: Date.now() + 60000 },
      round: 2,
      height: 40,
      payments_this_round: 3,
      autos_open: [{ id: 5, player: 'p2', name: 'Fazenda do Caio', round: 1, amount: 21600, paid: false, late_rounds: 1 }],
    }),
    players: [
      { id: 'p1', name: 'Padaria da Lu', sector: 'padaria', registered: true, balance: 1990000, suspended: false, mission_complete: true },
      { id: 'p2', name: 'Fazenda do Caio', sector: 'fazenda', registered: true, balance: 1500000, suspended: true, mission_complete: false },
      { id: 'p3', name: 'Novato', sector: 'fazenda', registered: false, balance: 2000000, suspended: false, mission_complete: false },
    ],
    invoices_open: 4,
    coinjoin: { entries: 2, deadline: Date.now() + 25000 },
    log: [
      { txid: 'aa', direction: 'out', amount: 60000, counterparty: 'Fazenda do Caio', memo: 'farinha e ovos', height: 40, round: 2, name: 'Padaria da Lu' },
    ],
    ...overrides,
  }
}

test('asks for the admin token once and stores it', async () => {
  const fetchMock = mockFetch([['GET', '/api/admin/room', makeRoom()]])
  render(<PresenterApp />)
  fireEvent.change(screen.getByLabelText('token do apresentador'), { target: { value: 'segredo' } })
  fireEvent.click(screen.getByText('entrar'))
  await screen.findByText('Avançar edição')
  expect(storage.get(ADMIN_KEY)).toBe('segredo')
  expect(fetchMock.find('GET', '/api/admin/room')[0].headers['X-Admin']).toBe('segredo')
  expect(FakeWebSocket.last().url).toContain('/ws/admin?token=segredo')
})

test('a rejected token goes back to the login', async () => {
  storage.set(ADMIN_KEY, 'errado')
  mockFetch([['GET', '/api/admin/room', { __status: 401, body: { error: 'token inválido' } }]])
  render(<PresenterApp />)
  await screen.findByText('Token recusado.')
  expect(storage.get(ADMIN_KEY)).toBeNull()
})

test('shows the buttons in order and the room state, refreshing on admin events', async () => {
  storage.set(ADMIN_KEY, 'adm')
  const fetchMock = mockFetch([['GET', '/api/admin/room', () => makeRoom()]])
  render(<PresenterApp />)
  await screen.findByText('Avançar edição')
  const labels = [
    'Abrir cadastro', 'Avançar edição', 'Abrir negociação', 'Fechar e apurar',
    'Cashback Cidadão', 'Receber cadastro das exchanges', 'Liberar relatório', 'Minerar bloco',
  ]
  const buttons = screen.getAllByRole('button').map((b) => b.textContent)
  const positions = labels.map((l) => buttons.indexOf(l))
  expect(positions.every((p) => p >= 0)).toBe(true)
  expect([...positions].sort((a, b) => a - b)).toEqual(positions)

  expect(screen.getAllByTestId('player')).toHaveLength(3)
  expect(screen.getByText('Padaria')).toBeTruthy()
  expect(screen.getByText('Fazenda')).toBeTruthy()
  expect(screen.getByText('suspenso')).toBeTruthy()
  expect(screen.getByText('sem cadastro')).toBeTruthy()
  expect(screen.getByText('missão completa')).toBeTruthy()
  expect(screen.getAllByTestId('auto')).toHaveLength(1)
  expect(screen.getByTestId('coinjoin').textContent).toContain('2 dentro')
  expect(screen.getAllByTestId('log-entry')).toHaveLength(1)
  expect(screen.getByText('Liberar relatório').disabled).toBe(true)

  const before = fetchMock.find('GET', '/api/admin/room').length
  act(() => FakeWebSocket.last().emit({ event: 'player_joined', player: 'p4' }))
  await waitFor(() => expect(fetchMock.find('GET', '/api/admin/room').length).toBe(before + 1))
})

test('buttons call the admin endpoints and report the result', async () => {
  storage.set(ADMIN_KEY, 'adm')
  const fetchMock = mockFetch([
    ['GET', '/api/admin/room', () => makeRoom()],
    ['POST', '/api/admin/advance', { phase: { phase: 'newspaper', round: 3 } }],
    ['POST', '/api/admin/open', { phase: { phase: 'negotiation', round: 3, deadline: 1 } }],
    ['POST', '/api/admin/close', { phase: { phase: 'reveal', round: 3 }, reveal: {} }],
    ['POST', '/api/admin/dust', { txid: 'dd' }],
    ['POST', '/api/admin/kyc', { players: ['p5'] }],
    ['POST', '/api/admin/mine', { height: 41 }],
    ['POST', '/api/admin/autos/5/forgive', {}],
    ['POST', '/api/admin/reset', {}],
    ['POST', '/api/admin/bots', { players: ['b1', 'b2'] }],
  ])
  vi.spyOn(window, 'confirm').mockReturnValue(true)
  render(<PresenterApp />)
  await screen.findByText('Avançar edição')

  fireEvent.click(screen.getByText('Avançar edição'))
  await screen.findByText('Avançar edição: fase: jornal, rodada 3')
  fireEvent.click(screen.getByText('Abrir negociação'))
  await screen.findByText(/Abrir negociação: fase: negociação/)
  fireEvent.click(screen.getByText('Fechar e apurar'))
  await screen.findByText(/Fechar e apurar: fase: apuração/)
  fireEvent.click(screen.getByText('Cashback Cidadão'))
  await screen.findByText('Cashback Cidadão: txid dd')
  fireEvent.click(screen.getByText('Receber cadastro das exchanges'))
  await screen.findByText('Cadastro das exchanges: 1 jogador(es)')
  fireEvent.click(screen.getByText('Minerar bloco'))
  await screen.findByText('Minerar bloco: bloco 41')
  fireEvent.click(screen.getByText('perdoar'))
  await screen.findByText('Perdoar auto 5: ok')
  fireEvent.click(screen.getByText('Abrir cadastro'))
  await screen.findByText('Abrir cadastro: ok')
  expect(window.confirm).toHaveBeenCalled()
  fireEvent.change(screen.getByLabelText('bots a adicionar'), { target: { value: '2' } })
  fireEvent.click(screen.getByText('adicionar bots'))
  await screen.findByText('Modo bots: 2 jogador(es)')
  expect(fetchMock.find('POST', '/api/admin/bots')[0].body).toEqual({ count: 2 })
  expect(fetchMock.find('POST', '/api/admin/autos/5/forgive')).toHaveLength(1)
})

test('config fields post numbers and show server errors', async () => {
  storage.set(ADMIN_KEY, 'adm')
  const fetchMock = mockFetch([
    ['GET', '/api/admin/room', () => makeRoom()],
    ['POST', '/api/admin/config', (body) => ({ negotiation_secs: body.negotiation_secs, icms_percent: 18, penalty_percent: 100, late_percent: 20, target_revenue: 120000 })],
    ['POST', '/api/admin/advance', { __status: 409, body: { error: 'negociação ainda aberta' } }],
  ])
  render(<PresenterApp />)
  await screen.findByText('Avançar edição')
  const timer = screen.getByLabelText('timer da negociação (s)')
  expect(timer.value).toBe('180')
  fireEvent.change(timer, { target: { value: '240' } })
  fireEvent.click(screen.getByText('salvar configuração'))
  await screen.findByText(/Configuração: ok/)
  expect(fetchMock.find('POST', '/api/admin/config')[0].body).toEqual({
    negotiation_secs: 240, icms_percent: 18, penalty_percent: 100, late_percent: 20, target_revenue: 120000,
  })
  fireEvent.click(screen.getByText('Avançar edição'))
  await screen.findByText('Avançar edição: negociação ainda aberta')
})

test('Liberar relatório is enabled on the reveal of round 8', async () => {
  storage.set(ADMIN_KEY, 'adm')
  mockFetch([
    ['GET', '/api/admin/room', () => makeRoom({ state: makeState({ phase: { phase: 'reveal', round: 8 }, round: 8 }) })],
    ['POST', '/api/admin/advance', { phase: { phase: 'final' } }],
  ])
  render(<PresenterApp />)
  const button = await screen.findByText('Liberar relatório')
  await waitFor(() => expect(button.disabled).toBe(false))
  fireEvent.click(button)
  await screen.findByText('Liberar relatório: fase: final')
})
