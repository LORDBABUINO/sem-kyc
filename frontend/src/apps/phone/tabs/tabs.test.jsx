import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react'
import Mission from './Mission.jsx'
import Wallet from './Wallet.jsx'
import Sell from './Sell.jsx'
import History from './History.jsx'
import Badge from './Badge.jsx'
import { mockFetch, clearStorage } from '../../../test/mocks.js'
import { makeMe, makeCoin, makeInvoice, makeMission } from '../../../test/fixtures.js'

beforeEach(() => clearStorage())
afterEach(() => vi.useRealTimers())

const noop = () => {}

test('Mission lists items with supplier, price and status, the timer and the bonus line', () => {
  const me = makeMe({
    mission: makeMission({
      items: [
        { item: 'farinha e ovos', status: { status: 'paid', txid: 'aa' }, price: 60000, supplier: makeMe().mission.items[0].supplier },
        { item: 'balcão e reforma', status: { status: 'expired' }, price: 40000, supplier: makeMe().mission.items[1].supplier },
      ],
      complete: true,
      bonus: 'bb',
    }),
  })
  render(<Mission me={me} novidades={[]} onDismiss={noop} onOpen={noop} onTab={noop} />)
  expect(screen.getAllByTestId('mission-item')).toHaveLength(2)
  expect(screen.getByText('pago')).toBeTruthy()
  expect(screen.getByText('expirado')).toBeTruthy()
  expect(screen.getByText('60.000 sats')).toBeTruthy()
  expect(screen.getByText('Fazenda')).toBeTruthy()
  expect(screen.getByLabelText('tempo restante').textContent).toMatch(/^1:5\d$|^2:00$/)
  expect(screen.getByText('bônus da cooperativa: chegou')).toBeTruthy()
  expect(screen.queryByText('Relatório')).toBeNull()
})

test('Mission shows the Relatório button only when available and never a finding', () => {
  const onOpen = vi.fn()
  render(<Mission me={makeMe({ report_available: true })} novidades={[]} onDismiss={noop} onOpen={onOpen} onTab={noop} />)
  fireEvent.click(screen.getByText('Relatório'))
  expect(onOpen).toHaveBeenCalledWith({ kind: 'report' })
  expect(document.body.textContent).not.toMatch(/severidade|finding|risco/i)
})

test('Mission novidade card for a wallet unlock switches to the wallet tab and can be dismissed', () => {
  const onTab = vi.fn()
  const onDismiss = vi.fn()
  render(<Mission me={makeMe()} novidades={['freeze']} onDismiss={onDismiss} onOpen={noop} onTab={onTab} />)
  expect(screen.getByTestId('novidade').textContent).toContain('não gaste, congele')
  fireEvent.click(screen.getByText('abrir carteira'))
  expect(onTab).toHaveBeenCalledWith('wallet')
  fireEvent.click(screen.getByText('ok'))
  expect(onDismiss).toHaveBeenCalledWith(0)
})

test('Wallet shows balance, coins with kind, origin from round 5, and gated actions', async () => {
  const exchangeCoin = makeCoin({ outpoint: 'dd'.repeat(32) + ':0', amount: 2000000, origin: { kind: 'exchange_withdrawal' } })
  const me = makeMe({
    round: 5,
    coins: [makeCoin(), exchangeCoin],
    player: { ...makeMe().player, unlocked: ['freeze', 'lightning', 'coin_join'] },
  })
  const fetchMock = mockFetch([
    ['POST', '/api/coins/freeze', () => makeMe()],
    ['POST', '/api/lightning', { out: 'x', inbound: 'y' }],
    ['POST', '/api/coinjoin', { entries: 2, deadline: Date.now() + 30000 }],
    ['GET', '/api/me', () => me],
  ])
  const onMe = vi.fn()
  render(<Wallet me={me} onMe={onMe} onOpen={noop} />)
  expect(screen.getByText('1.990.000 sats')).toBeTruthy()
  expect(screen.getByText(/segwit · saldo inicial/)).toBeTruthy()
  expect(screen.getByText(/saque de exchange/)).toBeTruthy()
  expect(screen.queryByText('Migrar pra Taproot')).toBeNull()

  fireEvent.click(screen.getByText('100.000 sats'))
  expect(screen.getByText('congelar')).toBeTruthy()
  expect(screen.queryByText('sacar via Lightning')).toBeNull()
  fireEvent.click(screen.getByText('congelar'))
  await waitFor(() => expect(onMe).toHaveBeenCalled())
  expect(fetchMock.find('POST', '/api/coins/freeze')[0].body).toEqual({ outpoint: makeCoin().outpoint, frozen: true })

  fireEvent.click(screen.getByText('2.000.000 sats'))
  fireEvent.click(screen.getByText('sacar via Lightning'))
  await screen.findByText(/Saque via Lightning feito/)
  fireEvent.click(screen.getByText('entrar no CoinJoin'))
  await screen.findByText(/Você entrou na sala: 2 dentro/)
  expect(fetchMock.find('POST', '/api/coinjoin')[0].body).toEqual({ outpoint: exchangeCoin.outpoint })
})

test('Wallet hides origin before round 5 and shows the coinjoin room', () => {
  const me = makeMe({ round: 3, coinjoin: { entries: 3, deadline: Date.now() + 20000 } })
  render(<Wallet me={me} onMe={noop} onOpen={noop} />)
  expect(screen.queryByText(/saldo inicial/)).toBeNull()
  expect(screen.getByTestId('coinjoin-room').textContent).toContain('3 dentro')
})

test('Wallet taproot migration warns when more than one coin is marked', async () => {
  const me = makeMe({ player: { ...makeMe().player, unlocked: ['taproot'] } })
  const fetchMock = mockFetch([
    ['POST', '/api/taproot', { txids: ['a', 'b'] }],
    ['GET', '/api/me', () => me],
  ])
  render(<Wallet me={me} onMe={noop} onOpen={noop} />)
  fireEvent.click(screen.getByText('escolher moedas'))
  fireEvent.click(screen.getByLabelText('migrar 100.000 sats'))
  expect(screen.queryByText(/Juntar liga as moedas/)).toBeNull()
  fireEvent.click(screen.getByLabelText('migrar 60.000 sats'))
  expect(screen.getByText(/Juntar liga as moedas/)).toBeTruthy()
  fireEvent.click(screen.getByText('uma transação por moeda'))
  await screen.findByText(/Migração enviada: 2/)
  expect(fetchMock.find('POST', '/api/taproot')[0].body).toEqual({
    outpoints: [makeMe().coins[0].outpoint, makeMe().coins[1].outpoint],
    together: false,
  })
})

test('Wallet gear opens the settings', () => {
  const onOpen = vi.fn()
  render(<Wallet me={makeMe()} onMe={noop} onOpen={onOpen} />)
  fireEvent.click(screen.getByLabelText('configurações'))
  expect(onOpen).toHaveBeenCalledWith({ kind: 'settings' })
})

test('Sell shows the catalog, creates an invoice and shows its QR and code, then flashes when paid', async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true })
  const invoice = makeInvoice({ code: 'ZZ99QQ', seller: 'p1', seller_name: 'Padaria da Lu', item: 'refeições da colheita', amount: 50000 })
  mockFetch([['POST', '/api/invoices', invoice]])
  const me = makeMe()
  const { rerender } = render(<Sell me={me} onMe={noop} />)
  expect(screen.getAllByText('50.000 sats')).toHaveLength(2)
  fireEvent.click(screen.getByText('refeições da colheita'))
  await screen.findByLabelText('código do pedido')
  expect(screen.getByLabelText('código do pedido').textContent).toBe('ZZ99QQ')
  expect(screen.getByRole('img', { name: 'QR do pedido' }).getAttribute('data-value')).toBe(invoice.uri)

  const paid = { ...invoice, status: { status: 'paid', buyer: 'p2', txid: '99'.repeat(32) } }
  const history = [{ txid: '99'.repeat(32), direction: 'in', amount: 50000, counterparty: 'Fazenda do Caio', memo: 'refeições da colheita', height: null, round: 1 }]
  rerender(<Sell me={makeMe({ invoices: [paid], history })} onMe={noop} />)
  expect(screen.getByText('recebido de Fazenda do Caio, 50.000 sats')).toBeTruthy()
  expect(screen.queryByLabelText('código do pedido')).toBeNull()
  act(() => vi.advanceTimersByTime(2100))
  expect(screen.queryByText(/recebido de/)).toBeNull()
})

test('Sell disables items without price and lists payjoin requests to answer', async () => {
  const request = makeInvoice({ code: 'PJ1234', payjoin: true, status: { status: 'payjoin_pending', buyer: 'p3' } })
  const fetchMock = mockFetch([
    ['POST', '/api/invoices/PJ1234/payjoin', { txid: 'ee' }],
    ['GET', '/api/me', () => makeMe()],
  ])
  const me = makeMe({ catalog: [{ name: 'pães', price: 0 }], payjoin_requests: [request] })
  const onMe = vi.fn()
  render(<Sell me={me} onMe={onMe} />)
  expect(screen.getByText('pães').closest('button').disabled).toBe(true)
  expect(screen.getByText('fora da negociação')).toBeTruthy()
  const card = screen.getByTestId('payjoin-request')
  expect(card.textContent).toContain('quer pagar junto')
  expect(screen.getByText('contribuir esta moeda').disabled).toBe(true)
  fireEvent.click(screen.getByLabelText('moeda de 60.000 sats'))
  fireEvent.click(screen.getByText('contribuir esta moeda'))
  await waitFor(() => expect(onMe).toHaveBeenCalled())
  expect(fetchMock.find('POST', '/api/invoices/PJ1234/payjoin')[0].body).toEqual({ contribute: makeMe().coins[1].outpoint })
})

test('Sell can decline a payjoin', async () => {
  const request = makeInvoice({ code: 'PJ1234', payjoin: true, status: { status: 'payjoin_pending' } })
  const fetchMock = mockFetch([
    ['POST', '/api/invoices/PJ1234/payjoin', { txid: 'ee' }],
    ['GET', '/api/me', () => makeMe()],
  ])
  render(<Sell me={makeMe({ payjoin_requests: [request] })} onMe={noop} />)
  fireEvent.click(screen.getByText('recusar (sai normal)'))
  await waitFor(() => expect(fetchMock.find('POST', '/api/invoices/PJ1234/payjoin').length).toBe(1))
  expect(fetchMock.find('POST', '/api/invoices/PJ1234/payjoin')[0].body).toEqual({ contribute: null })
})

test('History lists entries like a wallet and reveals the txid on tap', () => {
  render(<History me={makeMe()} />)
  expect(screen.getByLabelText('saída')).toBeTruthy()
  expect(screen.getByLabelText('entrada')).toBeTruthy()
  expect(screen.getByText('-10.000 sats')).toBeTruthy()
  expect(screen.getByText('+250.000 sats')).toBeTruthy()
  expect(screen.getByText('bloco 3')).toBeTruthy()
  expect(screen.getByText('na fila')).toBeTruthy()
  expect(screen.getByText('taxa de cadastro da vitrine')).toBeTruthy()
  expect(screen.queryByTestId('txid')).toBeNull()
  fireEvent.click(screen.getByText('DOE'))
  expect(screen.getByTestId('txid').textContent).toBe('11'.repeat(32))
})

test('Badge fills the screen with the sector color, icon and name', () => {
  render(<Badge me={makeMe()} />)
  const name = screen.getByText('Padaria da Lu')
  expect(name.parentElement.getAttribute('style')).toContain('#e0a030')
  expect(screen.getByText('Padaria')).toBeTruthy()
})
