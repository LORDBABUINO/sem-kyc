const CODE = /^[A-Za-z0-9]{6}$/

// Transforma o texto lido (QR ou digitado) no que o app sabe pagar.
export function parseScan(text, counterAddress) {
  const raw = String(text || '').trim()
  if (!raw) return { kind: 'empty' }
  if (CODE.test(raw)) return { kind: 'code', code: raw.toUpperCase() }

  if (/^bitcoin:/i.test(raw)) {
    const rest = raw.slice('bitcoin:'.length)
    const [address, query = ''] = rest.split('?')
    const params = new URLSearchParams(query)
    if (counterAddress && address === counterAddress) return { kind: 'counter', address }
    const direct = params.get('code') || params.get('invoice')
    if (direct && CODE.test(direct)) return { kind: 'code', code: direct.toUpperCase(), address }
    const pj = params.get('pj')
    if (pj) {
      const last = pj.replace(/\/+$/, '').split('/').pop()
      if (CODE.test(last)) return { kind: 'code', code: last.toUpperCase(), address }
    }
    return { kind: 'unknown', address }
  }

  return { kind: 'unknown' }
}
