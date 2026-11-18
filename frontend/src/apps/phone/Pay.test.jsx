import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react'
import Pay from './Pay.jsx'
import PayAuto from './PayAuto.jsx'
import { mockFetch, clearStorage } from '../../test/mocks.js'
import { makeMe, makeInvoice, makeCoin } from '../../test/fixtures.js'

beforeEach(() => clearStorage())
afterEach(() => vi.useRealTimers())

const noop = () => {}

function typeCode(code) {
  fireEvent.change(screen.getByLabelText('código do pedido'), { target: { value: code } })
  fireEvent.click(screen.getByText('buscar pedido'))
}

test('typing a code loads the invoice and pays automatically', async () => {
  const fetchMock = mockFetch([
    ['GET', '/api/invoices/ABC123', makeInvoice()],
    ['POST', '/api/invoices/ABC123/pay', { result: 'settled', txid: 'ff'.repeat(32) }],
  ])
  const onPaid = vi.fn()
  render(<Pay me={makeMe()} onClose={noop} onPaid={onPaid} onCounter={noop} />)
  expect(screen.getByText(/Aponte pro QR do vendedor/)).toBeTruthy()
  typeCode('ABC123')
  await screen.findByText('farinha e ovos')
  expect(screen.getByText('Fazenda do Caio')).toBeTruthy()
  expect(screen.getByText('60.000 sats')).toBeTruthy()
  expect(screen.queryByLabelText(/moeda de/)).toBeNull()
  expect(screen.queryByText('pagar junto (Payjoin)')).toBeNull()
  fireEvent.click(screen.getByText('pagar'))
  await screen.findByText('Pago.')
  expect(fetchMock.find('POST', '/api/invoices/ABC123/pay')[0].body).toEqual({})
  expect(onPaid).toHaveBeenCalled()
})

test('manual selection shows coins, total, change and the exact-coin hint', async () => {
  const me = makeMe({
    player: { ...makeMe().player, coin_selection: 'manual', unlocked: ['coin_control', 'odd_price'] },
    coins: [makeCoin(), makeCoin({ outpoint: 'bb'.repeat(32) + ':1', amount: 60000 }), makeCoin({ outpoint: 'ee'.repeat(32) + ':0', amount: 5000, frozen: true })],
  })
  const fetchMock = mockFetch([
    ['GET', '/api/invoices/ABC123', makeInvoice()],
    ['POST', '/api/invoices/ABC123/pay', { result: 'settled', txid: 'ff' }],
  ])
  render(<Pay me={me} initialCode="ABC123" onClose={noop} onPaid={noop} onCounter={noop} />)
  await screen.findByText('farinha e ovos')
  expect(screen.getAllByLabelText(/moeda de/)).toHaveLength(2)
  expect(screen.getByText('pague sem troco com esta')).toBeTruthy()
  expect(screen.getByText('pagar').disabled).toBe(true)
  fireEvent.click(screen.getByLabelText('moeda de 100.000 sats'))
  expect(screen.getByText('100.000 sats', { selector: 'strong' })).toBeTruthy()
  expect(screen.getByText('40.000 sats', { selector: 'strong' })).toBeTruthy()
  fireEvent.click(screen.getByText('pagar'))
  await screen.findByText('Pago.')
  expect(fetchMock.find('POST', '/api/invoices/ABC123/pay')[0].body).toEqual({ inputs: [makeCoin().outpoint] })
})

test('payjoin invoice with the unlock offers pagar junto as primary and waits for the seller', async () => {
  vi.useFakeTimers({ shouldAdvanceTime: true })
  let polls = 0
  const fetchMock = mockFetch([
    ['GET', '/api/invoices/ABC123', () => {
      polls += 1
      return makeInvoice({ payjoin: true, status: polls >= 3 ? { status: 'paid', txid: 'dd' } : { status: 'open' } })
    }],
    ['POST', '/api/invoices/ABC123/pay', { result: 'awaiting_seller' }],
  ])
  const me = makeMe({ player: { ...makeMe().player, unlocked: ['payjoin'] } })
  const onPaid = vi.fn()
  render(<Pay me={me} initialCode="ABC123" onClose={noop} onPaid={onPaid} onCounter={noop} />)
  const primary = await screen.findByText('pagar junto (Payjoin)')
  fireEvent.click(primary)
  await screen.findByText('Aguardando o vendedor.')
  expect(fetchMock.find('POST', '/api/invoices/ABC123/pay')[0].body).toEqual({ payjoin: true })
  await act(async () => {
    vi.advanceTimersByTime(2100)
  })
  await act(async () => {
    vi.advanceTimersByTime(2100)
  })
  await screen.findByText('Pago.')
  expect(onPaid).toHaveBeenCalledWith({ result: 'settled', txid: 'dd' })
})

test('without the payjoin unlock the payjoin invoice pays normally', async () => {
  mockFetch([['GET', '/api/invoices/ABC123', makeInvoice({ payjoin: true })]])
  render(<Pay me={makeMe()} initialCode="ABC123" onClose={noop} onPaid={noop} onCounter={noop} />)
  await screen.findByText('farinha e ovos')
  expect(screen.queryByText('pagar junto (Payjoin)')).toBeNull()
  expect(screen.getByText('pagar')).toBeTruthy()
})

test('shows server errors and unknown codes', async () => {
  mockFetch([['GET', '/api/invoices/NOPE00', { __status: 404, body: { error: 'pedido não encontrado' } }]])
  render(<Pay me={makeMe()} onClose={noop} onPaid={noop} onCounter={noop} />)
  typeCode('NOPE00')
  await screen.findByText('pedido não encontrado')
  typeCode('abc')
  expect(screen.getByText(/Digite o código de 6 caracteres/)).toBeTruthy()
})

test('PayAuto pays with manual coins and reports the txid', async () => {
  const me = makeMe({ player: { ...makeMe().player, coin_selection: 'manual', unlocked: ['coin_control'] } })
  const auto = { id: 7, player: 'p1', name: 'Padaria da Lu', round: 2, amount: 30000, paid: false, late_rounds: 1 }
  const fetchMock = mockFetch([['POST', '/api/autos/7/pay', { txid: 'ab'.repeat(32) }]])
  const onPaid = vi.fn()
  render(<PayAuto me={me} auto={auto} onClose={noop} onPaid={onPaid} />)
  expect(screen.getByText('Auto de Infração nº 7')).toBeTruthy()
  expect(screen.getByText(/multa de mora somada/)).toBeTruthy()
  expect(screen.getByText('pagar 30.000 sats').disabled).toBe(true)
  fireEvent.click(screen.getByLabelText('moeda de 60.000 sats'))
  fireEvent.click(screen.getByText('pagar 30.000 sats'))
  await screen.findByText('Auto quitado.')
  expect(fetchMock.find('POST', '/api/autos/7/pay')[0].body).toEqual({ inputs: [makeMe().coins[1].outpoint] })
  await waitFor(() => expect(onPaid).toHaveBeenCalled())
})

test('typing mode starts without the camera', () => {
  mockFetch([])
  render(<Pay me={makeMe()} typing onClose={noop} onPaid={noop} onCounter={noop} />)
  expect(screen.getByLabelText('código do pedido')).toBeTruthy()
  expect(document.querySelector('video')).toBeNull()
})

test('the exact coin is offered outside manual mode and pays with that coin', async () => {
  const me = makeMe({
    player: { ...makeMe().player, coin_selection: 'smallest_covering', unlocked: ['coin_control', 'odd_price'] },
    coins: [makeCoin(), makeCoin({ outpoint: 'bb'.repeat(32) + ':1', amount: 60000 })],
  })
  const fetchMock = mockFetch([
    ['GET', '/api/invoices/ABC123', makeInvoice()],
    ['POST', '/api/invoices/ABC123/pay', { result: 'settled', txid: 'ff' }],
  ])
  render(<Pay me={me} initialCode="ABC123" onClose={noop} onPaid={noop} onCounter={noop} />)
  await screen.findByText('farinha e ovos')
  fireEvent.click(screen.getByText('pague sem troco com a moeda de 60.000 sats'))
  await screen.findByText('Pago.')
  expect(fetchMock.find('POST', '/api/invoices/ABC123/pay')[0].body).toEqual({ inputs: ['bb'.repeat(32) + ':1'] })
})
