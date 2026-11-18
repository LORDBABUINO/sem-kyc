import { expect, test } from 'vitest'
import { parseScan } from './scan.js'

const COUNTER = 'bc1qguiche00000000000000000000000000000000'

test('accepts a typed 6-char code', () => {
  expect(parseScan(' abc123 ')).toEqual({ kind: 'code', code: 'ABC123' })
  expect(parseScan('')).toEqual({ kind: 'empty' })
})

test('recognises the counter QR', () => {
  expect(parseScan(`bitcoin:${COUNTER}`, COUNTER)).toEqual({ kind: 'counter', address: COUNTER })
  expect(parseScan(`bitcoin:${COUNTER}?amount=0.001`, COUNTER).kind).toBe('counter')
})

test('extracts the invoice code from the payjoin url or a code parameter', () => {
  const pj = parseScan('bitcoin:bc1qabc?amount=0.0006&label=farinha&pj=https%3A%2F%2Ff.x%2Fpj%2FXY12AB', COUNTER)
  expect(pj).toMatchObject({ kind: 'code', code: 'XY12AB' })
  const coded = parseScan('bitcoin:bc1qabc?amount=0.0006&code=QW34ER', COUNTER)
  expect(coded).toMatchObject({ kind: 'code', code: 'QW34ER' })
})

test('reports an unknown uri without code', () => {
  expect(parseScan('bitcoin:bc1qabc?amount=0.0006', COUNTER)).toEqual({ kind: 'unknown', address: 'bc1qabc' })
  expect(parseScan('hello world')).toEqual({ kind: 'unknown' })
})
