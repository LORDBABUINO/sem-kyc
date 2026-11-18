import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { render, screen, waitFor, act } from '@testing-library/react'
import ProjectorApp from './ProjectorApp.jsx'
import { mockFetch, installFakeWebSocket, FakeWebSocket } from '../../test/mocks.js'
import { makeState, makeClusters } from '../../test/fixtures.js'

const fakeContext = () =>
  new Proxy({}, { get: (target, key) => (key === 'canvas' ? null : () => undefined), set: () => true })

beforeEach(() => {
  installFakeWebSocket()
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(fakeContext)
})
afterEach(() => vi.restoreAllMocks())

function setup(state, clusters = makeClusters()) {
  let current = state
  const fetchMock = mockFetch([
    ['GET', '/api/state', () => current],
    ['GET', '/api/clusters', () => clusters],
  ])
  fetchMock.setState = (next) => {
    current = next
  }
  return fetchMock
}

test('lobby: giant join QR, counts and block', async () => {
  setup(makeState())
  render(<ProjectorApp />)
  const qr = await screen.findByRole('img', { name: 'QR de entrada' })
  expect(qr.getAttribute('data-value')).toBe('https://dever.exemplo/')
  expect(screen.getByText('9')).toBeTruthy()
  expect(screen.getByText('12')).toBeTruthy()
  expect(screen.getByText('estabelecimentos cadastrados')).toBeTruthy()
  expect(screen.queryByTestId('debt-ticker')).toBeNull()
  expect(FakeWebSocket.last().url).toContain('/ws/projector')
})

test('newspaper edition 1: masthead, headline, edital and registration footer', async () => {
  setup(makeState({ phase: { phase: 'newspaper', round: 1 }, round: 1, height: 21 }))
  render(<ProjectorApp />)
  await screen.findByText('DOE publica o saldo de todas as lojas da cidade')
  expect(screen.getByText('Rodada 1')).toBeTruthy()
  expect(screen.getByText('Bloco 21')).toBeTruthy()
  expect(screen.getByText(/Presunção|multa de ofício de 100%/)).toBeTruthy()
  expect(screen.getByText('Cadastro concluído: 9 estabelecimentos.')).toBeTruthy()
})

test('newspaper edition 3 shows the previous round reveal and the debt ticker', async () => {
  const autos = [
    { id: 1, player: 'p1', name: 'Padaria da Lu', round: 2, amount: 43200, paid: true, late_rounds: 0 },
    { id: 2, player: 'p2', name: 'Fazenda do Caio', round: 2, amount: 21600, paid: false, late_rounds: 0 },
  ]
  setup(
    makeState({
      phase: { phase: 'newspaper', round: 3 },
      round: 3,
      reveal: {
        round: 2,
        headline: { round: 2, player: 'p1', finding: 'CIOH' },
        headline_name: 'Padaria da Lu',
        autos,
        assessed: 64800,
        collected: 43200,
        payments: 14,
        known: { attributed_operations: 3, real_operations: 4, linked_addresses: 7, counterparties: ['Fazenda do Caio'] },
      },
      autos_open: [autos[1]],
      totals: { assessed: 64800, collected: 43200 },
    }),
  )
  render(<ProjectorApp />)
  await screen.findByText('DOE mapeia o troco e descobre fornecedores')
  expect(screen.getByText(/2 estabelecimentos autuados pelo Portaria DOE nº 2\. ICMS lançado: 64\.800 sats, quitado: 43\.200 sats\./)).toBeTruthy()
  const names = screen.getByTestId('footer-names')
  expect(names.textContent).toContain('Padaria da Lu')
  expect(names.textContent).toContain('quitado')
  expect(names.textContent).toContain('em aberto')
  const ticker = screen.getByTestId('debt-ticker')
  expect(ticker.textContent).toContain('Fazenda do Caio')
  expect(ticker.textContent).toContain('21.600 sats')
  expect(screen.getByRole('img', { name: 'QR do guichê do ICMS' }).getAttribute('data-value')).toBe('bitcoin:bc1qguiche00000000000000000000000000000000')
})

test('negotiation: top bar with timer, payments, block, and the graph refetches clusters on tx', async () => {
  const fetchMock = setup(
    makeState({ phase: { phase: 'negotiation', round: 2, deadline: Date.now() + 90000 }, round: 2, payments_this_round: 5, height: 33 }),
  )
  render(<ProjectorApp />)
  await screen.findByTestId('cluster-graph')
  expect(screen.getByLabelText('tempo restante').textContent).toMatch(/^1:[23]\d$/)
  expect(screen.getByText('5')).toBeTruthy()
  expect(screen.getByText('33')).toBeTruthy()
  const before = fetchMock.find('GET', '/api/clusters').length
  act(() => FakeWebSocket.last().emit({ event: 'tx', txid: 'aa', clusters: [{ event: 'merged', absorbed: 'r2', into: 'r1', cause: 'common_input', txid: 'aa' }] }))
  await waitFor(() => expect(fetchMock.find('GET', '/api/clusters').length).toBe(before + 1))
  const beforeState = fetchMock.find('GET', '/api/state').length
  act(() => FakeWebSocket.last().emit({ event: 'payments', round: 2, count: 6 }))
  await waitFor(() => expect(fetchMock.find('GET', '/api/state').length).toBe(beforeState + 1))
  expect(fetchMock.find('GET', '/api/clusters').length).toBe(before + 1)
})

test('reveal: state card, autuados with amounts, big counter QR and totals', async () => {
  const autos = [{ id: 4, player: 'p1', name: 'Padaria da Lu', round: 1, amount: 21600, paid: false, late_rounds: 0 }]
  setup(
    makeState({
      phase: { phase: 'reveal', round: 1 },
      round: 1,
      reveal: {
        round: 1,
        headline: { round: 1, player: 'p1', finding: { type: 'ADDRESS_REUSE', severity: 'high' } },
        headline_name: 'Padaria da Lu',
        autos,
        assessed: 21600,
        collected: 0,
        payments: 9,
        known: { attributed_operations: 2, real_operations: 2, linked_addresses: 3, counterparties: ['Fazenda do Caio', 'Construtora Pedra'] },
      },
      autos_open: autos,
      totals: { assessed: 21600, collected: 0 },
    }),
  )
  render(<ProjectorApp />)
  const card = await screen.findByTestId('state-knows')
  expect(card.textContent).toContain('Padaria da Lu')
  expect(card.textContent).toContain('2 de 2')
  expect(card.textContent).toContain('Fazenda do Caio, Construtora Pedra')
  expect(screen.getByText('Reuso de endereço')).toBeTruthy()
  expect(screen.getAllByTestId('reveal-auto')).toHaveLength(1)
  expect(screen.getAllByText('21.600 sats').length).toBeGreaterThan(0)
  expect(screen.getByRole('img', { name: 'QR grande do guichê' })).toBeTruthy()
  expect(screen.getByTestId('debt-ticker')).toBeTruthy()
})

test('final: headline and top 3 standings', async () => {
  setup(
    makeState({
      phase: { phase: 'final' },
      round: 8,
      standings: [
        { player: 'p1', name: 'Padaria da Lu', balance: 2400000 },
        { player: 'p2', name: 'Fazenda do Caio', balance: 2100000 },
        { player: 'p3', name: 'Gráfica Boa', balance: 1900000 },
        { player: 'p4', name: 'Quarto', balance: 100 },
      ],
    }),
  )
  render(<ProjectorApp />)
  await screen.findByText('DOE liberta os arquivos')
  const standings = screen.getAllByTestId('standing')
  expect(standings).toHaveLength(3)
  expect(screen.getByText('1º').parentElement.textContent).toContain('Padaria da Lu')
  expect(screen.queryByText('Quarto')).toBeNull()
  expect(screen.getByText('Publique sp1.')).toBeTruthy()
})

test('phase event switches scene after refetch', async () => {
  const fetchMock = setup(makeState())
  render(<ProjectorApp />)
  await screen.findByRole('img', { name: 'QR de entrada' })
  fetchMock.setState(makeState({ phase: { phase: 'newspaper', round: 1 }, round: 1 }))
  act(() => FakeWebSocket.last().emit({ event: 'phase', phase: { phase: 'newspaper', round: 1 } }))
  await screen.findByText('DOE publica o saldo de todas as lojas da cidade')
})
