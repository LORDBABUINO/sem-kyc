import { beforeEach, expect, test, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import Settings from './Settings.jsx'
import Report from './Report.jsx'
import { mockFetch, clearStorage } from '../../test/mocks.js'
import { makeMe } from '../../test/fixtures.js'

beforeEach(() => clearStorage())

test('Settings disables options until unlocked and posts the change', async () => {
  const me = makeMe({ player: { ...makeMe().player, unlocked: ['coin_control'] } })
  const fetchMock = mockFetch([['POST', '/api/settings', (body) => makeMe({ player: { ...me.player, ...body } })]])
  const onMe = vi.fn()
  render(<Settings me={me} onClose={() => {}} onMe={onMe} />)
  expect(screen.getByLabelText('vitrine (endereço fixo)').checked).toBe(true)
  expect(screen.getByLabelText('endereço novo a cada venda').disabled).toBe(true)
  expect(screen.getByLabelText('quebrado').disabled).toBe(true)
  expect(screen.getByLabelText('manual').disabled).toBe(false)
  expect(screen.getByLabelText('a menor moeda que cobre').disabled).toBe(false)
  expect(screen.queryByText('recadastrar vitrine como sp1')).toBeNull()
  fireEvent.click(screen.getByLabelText('manual'))
  await waitFor(() => expect(onMe).toHaveBeenCalled())
  expect(fetchMock.find('POST', '/api/settings')[0].body).toEqual({ coin_selection: 'manual' })
  expect(onMe.mock.calls[0][0].player.coin_selection).toBe('manual')
})

test('Settings offers the sp1 storefront once silent payments unlock', async () => {
  const me = makeMe({ player: { ...makeMe().player, unlocked: ['silent_payments', 'payjoin', 'taproot'] } })
  const fetchMock = mockFetch([
    ['POST', '/api/storefront/silent', { sp: 'sp1qexample' }],
    ['GET', '/api/me', () => makeMe({ player: { ...me.player, storefront_is_silent: true, silent_address: 'sp1qexample' } })],
  ])
  const onMe = vi.fn()
  render(<Settings me={me} onClose={() => {}} onMe={onMe} />)
  expect(screen.getByLabelText('Silent Payments (sp1)').disabled).toBe(false)
  fireEvent.click(screen.getByText('recadastrar vitrine como sp1'))
  await screen.findByText('sp1qexample')
  expect(fetchMock.find('POST', '/api/storefront/silent').length).toBe(1)
  expect(onMe).toHaveBeenCalled()
})

test('Report renders findings as cards with badge, severity, description and correction', async () => {
  mockFetch([
    ['GET', '/api/report', {
      stats: { transactions: 14 },
      findings: [
        { type: 'ADDRESS_REUSE', severity: 'high', description: 'A vitrine recebeu 3 vezes.', details: { address: 'bc1qvitrine' }, correction: 'Gere um endereço novo a cada venda.' },
        { type: 'CIOH', severity: 'medium', description: 'Duas moedas gastas juntas.', details: null, correction: 'Uma moeda por pagamento.' },
      ],
      warnings: [{ type: 'DORMANT_UTXOS', message: 'Moedas paradas há 5 rodadas.' }],
      summary: 'Dossiê de Padaria da Lu.',
    }],
  ])
  render(<Report onClose={() => {}} />)
  await screen.findByText('Dossiê de Padaria da Lu.')
  expect(screen.getAllByTestId('finding')).toHaveLength(2)
  expect(screen.getByText('Reuso de endereço')).toBeTruthy()
  expect(screen.getByText('alta')).toBeTruthy()
  expect(screen.getByText('Gere um endereço novo a cada venda.')).toBeTruthy()
  expect(screen.getByText('UTXOs dormentes')).toBeTruthy()
  expect(screen.getByText('Moedas paradas há 5 rodadas.')).toBeTruthy()
})

test('Report shows the 403 message before the final edition', async () => {
  mockFetch([['GET', '/api/report', { __status: 403, body: { error: 'relatório ainda não liberado' } }]])
  render(<Report onClose={() => {}} />)
  await screen.findByText('relatório ainda não liberado')
})
