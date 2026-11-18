import { expect, test } from 'vitest'
import { reconcile, nodeLabel, nodeColors, ghostPosition, findRootFor, fitCamera, nodeRadius, MERGE_MS, UNKNOWN_COLOR, STRIPE_COLOR } from './graphModel.js'
import { makeClusters } from '../../test/fixtures.js'

const empty = { nodes: [], ghosts: [] }

test('labels: name when known, "ou" with question mark on conflict, nothing when unknown', () => {
  const data = makeClusters()
  expect(nodeLabel(data.clusters[0])).toBe('Padaria da Lu')
  expect(nodeLabel(data.clusters[1])).toBe('')
  expect(nodeLabel(data.clusters[2])).toBe('Padaria da Lu ou Fazenda do Caio?')
})

test('colors: sector color, grey for unknown, two stripes on conflict', () => {
  const data = makeClusters()
  expect(nodeColors(data.clusters[0])).toEqual(['#e0a030'])
  expect(nodeColors(data.clusters[1])).toEqual([UNKNOWN_COLOR])
  expect(nodeColors(data.clusters[2])).toEqual(['#4caf50', STRIPE_COLOR])
})

test('radius grows with balance and the biggest cluster is the biggest node', () => {
  expect(nodeRadius(0, 100)).toBe(14)
  expect(nodeRadius(100, 100)).toBe(60)
  expect(nodeRadius(25, 100)).toBeGreaterThan(nodeRadius(10, 100))
})

test('reconcile keeps positions of existing nodes and builds links', () => {
  const first = reconcile(empty, makeClusters(), { now: 1000 })
  expect(first.nodes).toHaveLength(3)
  expect(first.links).toHaveLength(1)
  expect(first.links[0].addedAt).toBe(1000)
  first.nodes[0].x = 123
  first.nodes[0].y = 456
  const second = reconcile(first, makeClusters(), { now: 2000 })
  expect(second.nodes[0].x).toBe(123)
  expect(second.nodes[0].y).toBe(456)
  expect(second.ghosts).toHaveLength(0)
})

test('an absorbed cluster becomes a ghost sliding into the cluster that now holds its addresses', () => {
  const seenEdges = new Map()
  const first = reconcile(empty, makeClusters(), { now: 1000, seenEdges })
  const grey = first.nodes.find((n) => n.root === 'r2')
  grey.x = 10
  grey.y = 20
  const merged = makeClusters()
  merged.clusters = [
    { ...merged.clusters[0], addresses: ['bc1qa', 'bc1qb', 'bc1qc'], balance: 1560000 },
    merged.clusters[2],
  ]
  merged.edges = []
  const second = reconcile(first, merged, { now: 5000, seenEdges })
  expect(second.nodes).toHaveLength(2)
  expect(second.ghosts).toHaveLength(1)
  const ghost = second.ghosts[0]
  expect(ghost.target.root).toBe('r1')
  expect(ghostPosition(ghost, 5000)).toMatchObject({ x: 10, y: 20, t: 0 })
  const end = ghostPosition(ghost, 5000 + MERGE_MS)
  expect(end.x).toBeCloseTo(ghost.target.x)
  expect(end.y).toBeCloseTo(ghost.target.y)
  expect(end.t).toBe(1)
})

test('a brand-new cluster that inherits addresses starts where they were', () => {
  const first = reconcile(empty, makeClusters(), { now: 1000 })
  const old = first.nodes.find((n) => n.root === 'r1')
  old.x = 77
  old.y = 88
  const relabeled = makeClusters()
  relabeled.clusters[0] = { ...relabeled.clusters[0], root: 'r9' }
  relabeled.edges = []
  const second = reconcile(first, relabeled, { now: 1500 })
  const fresh = second.nodes.find((n) => n.root === 'r9')
  expect(fresh.x).toBe(77)
  expect(fresh.y).toBe(88)
})

test('findRootFor locates the headline cluster by player id or name', () => {
  const data = makeClusters()
  expect(findRootFor(data, { playerId: 'p2' })).toBe('r3')
  expect(findRootFor(data, { name: 'Padaria da Lu' })).toBe('r1')
  expect(findRootFor(data, { playerId: 'zz' })).toBeNull()
})

test('fitCamera frames all nodes or zooms on the focused one', () => {
  const nodes = [
    { root: 'a', x: 0, y: 0, r: 10 },
    { root: 'b', x: 200, y: 100, r: 10 },
  ]
  const all = fitCamera(nodes, 1000, 600)
  expect(all.x).toBe(100)
  expect(all.y).toBe(50)
  expect(all.k).toBeGreaterThan(1)
  const focus = fitCamera(nodes, 1000, 600, 'b')
  expect(focus).toEqual({ x: 200, y: 100, k: 2.2 })
})
