export const KIND_LABELS = { legacy: 'legacy', segwit: 'segwit', taproot: 'Taproot' }

export function kindLabel(kind) {
  return KIND_LABELS[kind] || kind || ''
}

export function originLabel(origin) {
  if (!origin) return ''
  switch (origin.kind) {
    case 'initial':
      return 'saldo inicial'
    case 'exchange_withdrawal':
      return 'saque de exchange'
    case 'sale':
      return origin.from ? `venda pra ${origin.from}` : 'venda'
    case 'change':
      return 'troco'
    case 'bonus':
      return 'bônus da cooperativa'
    case 'dust':
      return 'cashback'
    case 'lightning':
      return 'saque via Lightning'
    case 'coin_join':
      return 'CoinJoin'
    case 'migration':
      return 'migração pra Taproot'
    case 'tax':
      return 'guichê do ICMS'
    default:
      return origin.kind || ''
  }
}

export function spendable(coins) {
  return (coins || []).filter((c) => !c.frozen)
}

export function sumAmount(coins) {
  return coins.reduce((acc, c) => acc + (c.amount || 0), 0)
}

export const ORIGIN_VISIBLE_FROM_ROUND = 5

export function showsOrigin(me) {
  return (me && me.round >= ORIGIN_VISIBLE_FROM_ROUND) || false
}

export function hasUnlock(me, unlock) {
  return Boolean(me && me.player && me.player.unlocked && me.player.unlocked.includes(unlock))
}

export function openAutos(me) {
  return ((me && me.autos) || []).filter((a) => !a.paid)
}
