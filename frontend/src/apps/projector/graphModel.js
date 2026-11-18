// Estado puro do grafo de clusters, separado do canvas pra poder testar.
export const MERGE_MS = 500
export const PULSE_MS = 1500
export const UNKNOWN_COLOR = '#5b6270'
export const STRIPE_COLOR = '#e8e8e8'

export function nodeRadius(balance, maxBalance) {
  const ratio = maxBalance > 0 ? Math.sqrt(Math.max(0, balance) / maxBalance) : 0
  return 14 + 46 * ratio
}

export function identityKind(cluster) {
  return (cluster.identity && cluster.identity.identity) || 'unknown'
}

export function nodeLabel(cluster) {
  const kind = identityKind(cluster)
  const names = cluster.names || []
  if (kind === 'known') return names[0] || ''
  if (kind === 'conflict') return names.length ? `${names.join(' ou ')}?` : ''
  return ''
}

export function nodeColors(cluster) {
  const kind = identityKind(cluster)
  if (kind === 'unknown' || !cluster.color) return [UNKNOWN_COLOR]
  if (kind === 'conflict') return [cluster.color, STRIPE_COLOR]
  return [cluster.color]
}

function randomAround(cx, cy, radius) {
  const angle = Math.random() * Math.PI * 2
  return { x: cx + Math.cos(angle) * radius, y: cy + Math.sin(angle) * radius }
}

// Casa a lista nova de clusters com os nós anteriores: mantém posições, cria fantasmas pra
// quem foi absorvido e marca as arestas novas pra pulsar.
export function reconcile(previous, data, { now = Date.now(), width = 1280, height = 720, seenEdges = new Map() } = {}) {
  const prevByRoot = new Map(previous.nodes.map((n) => [n.root, n]))
  const prevByAddress = new Map()
  for (const node of previous.nodes) for (const addr of node.addresses || []) prevByAddress.set(addr, node)

  const clusters = data.clusters || []
  const maxBalance = clusters.reduce((m, c) => Math.max(m, c.balance || 0), 0)
  const cx = width / 2
  const cy = height / 2
  const periphery = Math.min(width, height) * 0.4

  const nodes = clusters.map((cluster) => {
    const kind = identityKind(cluster)
    const known = kind !== 'unknown'
    const base = {
      root: cluster.root,
      addresses: cluster.addresses || [],
      cluster,
      kind,
      known,
      label: nodeLabel(cluster),
      colors: nodeColors(cluster),
      balance: cluster.balance || 0,
      r: nodeRadius(cluster.balance || 0, maxBalance),
    }
    const prev = prevByRoot.get(cluster.root)
    if (prev) return Object.assign(prev, base)
    const origin = base.addresses.map((a) => prevByAddress.get(a)).find(Boolean)
    const start = origin ? { x: origin.x, y: origin.y } : randomAround(cx, cy, known ? periphery * 0.3 : periphery)
    return { ...base, x: start.x, y: start.y, vx: 0, vy: 0 }
  })

  const byRoot = new Map(nodes.map((n) => [n.root, n]))
  const byAddress = new Map()
  for (const node of nodes) for (const addr of node.addresses) byAddress.set(addr, node)

  const ghosts = previous.nodes
    .filter((n) => !byRoot.has(n.root))
    .map((n) => {
      const target = (n.addresses || []).map((a) => byAddress.get(a)).find(Boolean) || null
      return { from: { x: n.x, y: n.y }, target, r: n.r, colors: n.colors, start: now }
    })

  const links = []
  for (const edge of data.edges || []) {
    if (!byRoot.has(edge.from) || !byRoot.has(edge.to)) continue
    if (!seenEdges.has(edge.txid)) seenEdges.set(edge.txid, now)
    links.push({ source: edge.from, target: edge.to, txid: edge.txid, amount: edge.amount, addedAt: seenEdges.get(edge.txid) })
  }

  return { nodes, links, ghosts: [...previous.ghosts.filter((g) => now - g.start < MERGE_MS), ...ghosts] }
}

export function ghostPosition(ghost, now) {
  const t = Math.min(1, (now - ghost.start) / MERGE_MS)
  const eased = 1 - (1 - t) * (1 - t)
  if (!ghost.target) return { x: ghost.from.x, y: ghost.from.y, t, alpha: 1 - eased }
  return {
    x: ghost.from.x + (ghost.target.x - ghost.from.x) * eased,
    y: ghost.from.y + (ghost.target.y - ghost.from.y) * eased,
    t,
    alpha: 1 - eased * 0.7,
  }
}

export function findRootFor(data, { playerId, name }) {
  if (!data) return null
  for (const cluster of data.clusters || []) {
    const players = (cluster.identity && cluster.identity.players) || []
    if (playerId && players.includes(playerId)) return cluster.root
    if (name && (cluster.names || []).includes(name)) return cluster.root
  }
  return null
}

export function fitCamera(nodes, width, height, focusRoot) {
  if (!nodes.length) return { x: width / 2, y: height / 2, k: 1 }
  if (focusRoot) {
    const node = nodes.find((n) => n.root === focusRoot)
    if (node) return { x: node.x, y: node.y, k: 2.2 }
  }
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const n of nodes) {
    minX = Math.min(minX, n.x - n.r)
    minY = Math.min(minY, n.y - n.r)
    maxX = Math.max(maxX, n.x + n.r)
    maxY = Math.max(maxY, n.y + n.r)
  }
  const bw = Math.max(1, maxX - minX)
  const bh = Math.max(1, maxY - minY)
  const k = Math.min(2, Math.max(0.3, Math.min((width * 0.85) / bw, (height * 0.75) / bh)))
  return { x: (minX + maxX) / 2, y: (minY + maxY) / 2, k }
}
