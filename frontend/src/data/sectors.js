export const SECTOR_ICONS = {
  padaria: '🥖',
  fazenda: '🌾',
  energia: '☀️',
  transportadora: '🚚',
  loja: '📱',
  construtora: '🏗️',
  consultoria: '📋',
  grafica: '🖨️',
}

export const SECTOR_NAMES = {
  padaria: 'Padaria',
  fazenda: 'Fazenda',
  energia: 'Energia solar',
  transportadora: 'Transportadora',
  loja: 'Loja de eletrônicos',
  construtora: 'Construtora',
  consultoria: 'Consultoria',
  grafica: 'Gráfica',
}

export const SECTOR_ORDER = Object.keys(SECTOR_NAMES)

export function sectorIcon(sector) {
  const key = typeof sector === 'string' ? sector : sector && sector.sector
  return SECTOR_ICONS[key] || '🏪'
}

export function sectorName(sector) {
  if (!sector) return ''
  if (typeof sector === 'string') return SECTOR_NAMES[sector] || sector
  return sector.name || SECTOR_NAMES[sector.sector] || sector.sector || ''
}
