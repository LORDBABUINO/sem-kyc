import { expect, test } from 'vitest'
import { EDITIONS, FINAL_EDITION, editionFor } from './editions.js'
import { UNLOCKS } from './unlocks.js'

test('has the eight editions with headline, subheadline and edital', () => {
  for (let round = 1; round <= 8; round += 1) {
    const edition = editionFor(round)
    expect(edition.headline.length).toBeGreaterThan(10)
    expect(edition.subheadline.length).toBeGreaterThan(10)
    expect(edition.edital.length).toBeGreaterThan(10)
  }
  expect(Object.keys(EDITIONS)).toHaveLength(8)
  expect(editionFor(9)).toBeNull()
  expect(FINAL_EDITION.headline).toBe('DOE liberta os arquivos')
  expect(FINAL_EDITION.takeaways).toHaveLength(6)
})

test('every unlock has exactly one sentence and a target', () => {
  const names = [
    'fresh_address', 'coin_control', 'odd_price', 'freeze', 'lightning',
    'coin_join', 'taproot', 'payjoin', 'silent_payments', 'report',
  ]
  for (const name of names) {
    const info = UNLOCKS[name]
    expect(info, name).toBeTruthy()
    expect(info.sentence.split(/[.!?](\s|$)/).filter((s) => s.trim()).length).toBe(1)
    expect(['settings', 'wallet', 'report']).toContain(info.target)
  }
})

test('texts never use em dashes', () => {
  const all = JSON.stringify(EDITIONS) + JSON.stringify(FINAL_EDITION) + JSON.stringify(UNLOCKS)
  expect(all.includes('—')).toBe(false)
})
