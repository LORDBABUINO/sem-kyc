export function formatNumber(n) {
  const value = Math.trunc(Number(n) || 0)
  const sign = value < 0 ? '-' : ''
  return sign + String(Math.abs(value)).replace(/\B(?=(\d{3})+(?!\d))/g, '.')
}

export function formatSats(n) {
  return `${formatNumber(n)} sats`
}

export function parseDeadline(deadline) {
  if (deadline == null) return null
  if (typeof deadline === 'number') return deadline < 1e12 ? deadline * 1000 : deadline
  const asNumber = Number(deadline)
  if (!Number.isNaN(asNumber)) return parseDeadline(asNumber)
  const parsed = Date.parse(deadline)
  return Number.isNaN(parsed) ? null : parsed
}

export function secondsLeft(deadline, now = Date.now()) {
  const at = parseDeadline(deadline)
  if (at == null) return null
  return Math.max(0, Math.ceil((at - now) / 1000))
}

export function formatCountdown(seconds) {
  if (seconds == null) return '--:--'
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${String(s).padStart(2, '0')}`
}

export function shortId(txid, size = 8) {
  if (!txid) return ''
  return txid.length > size * 2 + 1 ? `${txid.slice(0, size)}…${txid.slice(-size)}` : txid
}
