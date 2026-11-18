import { expect, test } from 'vitest'
import { formatSats, formatNumber, parseDeadline, secondsLeft, formatCountdown, shortId } from './format.js'

test('formats sats with pt-BR thousands separator', () => {
  expect(formatSats(49731)).toBe('49.731 sats')
  expect(formatSats(2000000)).toBe('2.000.000 sats')
  expect(formatSats(546)).toBe('546 sats')
  expect(formatNumber(-1234)).toBe('-1.234')
  expect(formatSats(undefined)).toBe('0 sats')
})

test('parses deadlines as epoch seconds, millis or ISO strings', () => {
  expect(parseDeadline(1700000000)).toBe(1700000000000)
  expect(parseDeadline(1700000000000)).toBe(1700000000000)
  expect(parseDeadline('2026-01-01T00:00:00Z')).toBe(Date.parse('2026-01-01T00:00:00Z'))
  expect(parseDeadline(null)).toBeNull()
})

test('counts down and never goes negative', () => {
  const now = 1700000000000
  expect(secondsLeft(now + 65000, now)).toBe(65)
  expect(secondsLeft(now - 5000, now)).toBe(0)
  expect(formatCountdown(65)).toBe('1:05')
  expect(formatCountdown(null)).toBe('--:--')
})

test('shortens txids', () => {
  expect(shortId('ab'.repeat(32))).toBe('abababab…abababab')
  expect(shortId('abc')).toBe('abc')
})
