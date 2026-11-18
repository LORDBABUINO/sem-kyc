export const PADARIA = { sector: 'padaria', name: 'Padaria', color: '#e0a030' }
export const FAZENDA = { sector: 'fazenda', name: 'Fazenda', color: '#4caf50' }
export const CONSTRUTORA = { sector: 'construtora', name: 'Construtora', color: '#9e7b5a' }

export function makeCoin(overrides = {}) {
  return {
    outpoint: 'aa'.repeat(32) + ':0',
    address: 'bc1qpadaria000000000000000000000000000001',
    amount: 100000,
    kind: 'segwit',
    frozen: false,
    origin: { kind: 'initial' },
    round: 0,
    ...overrides,
  }
}

export function makeMission(overrides = {}) {
  return {
    round: 1,
    text: 'As portas abriram. Falta o básico: farinha e ovos da Fazenda e balcão e reforma com a Construtora.',
    items: [
      { item: { name: 'farinha e ovos', supplier: 'fazenda' }, status: { status: 'pending' }, price: 60000, supplier: FAZENDA },
      { item: { name: 'balcão e reforma', supplier: 'construtora' }, status: { status: 'pending' }, price: 40000, supplier: CONSTRUTORA },
    ],
    bonus: null,
    complete: false,
    ...overrides,
  }
}

export function makeMe(overrides = {}) {
  const mission = makeMission()
  return {
    player: {
      id: 'p1',
      name: 'Padaria da Lu',
      sector: PADARIA,
      storefront: 'bc1qvitrine0000000000000000000000000000001',
      registered: true,
      storefront_is_silent: false,
      receive_mode: 'storefront',
      price_style: 'round',
      coin_selection: 'automatic',
      unlocked: [],
      taproot: false,
      silent_address: null,
    },
    phase: { phase: 'negotiation', round: 1, deadline: Date.now() + 120000 },
    round: 1,
    height: 12,
    balance: 1990000,
    coins: [
      makeCoin(),
      makeCoin({ outpoint: 'bb'.repeat(32) + ':1', amount: 60000, kind: 'legacy', address: '1Padaria0000000000000000000000002' }),
      makeCoin({ outpoint: 'cc'.repeat(32) + ':0', amount: 250000, origin: { kind: 'sale', from: 'Fazenda do Caio' }, round: 1 }),
    ],
    mission,
    missions: [mission],
    catalog: [
      { name: 'refeições da colheita', price: 50000 },
      { name: 'pães do canteiro', price: 50000 },
    ],
    invoices: [],
    payjoin_requests: [],
    history: [
      {
        txid: '11'.repeat(32),
        direction: 'out',
        amount: 10000,
        counterparty: 'DOE',
        memo: 'taxa de cadastro da vitrine',
        height: 3,
        round: 0,
      },
      {
        txid: '22'.repeat(32),
        direction: 'in',
        amount: 250000,
        counterparty: 'Fazenda do Caio',
        memo: 'refeições da colheita',
        height: null,
        round: 1,
      },
    ],
    autos: [],
    suspended: false,
    debt: 0,
    counter_address: 'bc1qguiche00000000000000000000000000000000',
    coinjoin: null,
    report_available: false,
    ...overrides,
  }
}

export function makeInvoice(overrides = {}) {
  return {
    code: 'ABC123',
    seller: 'p2',
    seller_name: 'Fazenda do Caio',
    seller_sector: FAZENDA,
    item: { name: 'farinha e ovos', supplier: 'fazenda' },
    amount: 60000,
    display: 'bc1qfazenda000000000000000000000000000002',
    uri: 'bitcoin:bc1qfazenda000000000000000000000000000002?amount=0.0006&label=farinha%20e%20ovos',
    payjoin: false,
    round: 1,
    status: { status: 'open' },
    ...overrides,
  }
}

export function makeState(overrides = {}) {
  return {
    phase: { phase: 'lobby' },
    round: 0,
    height: 4,
    players: 12,
    registered: 9,
    counter_address: 'bc1qguiche00000000000000000000000000000000',
    join_url: 'https://dever.exemplo/',
    reveal: null,
    autos_open: [],
    totals: { assessed: 0, collected: 0 },
    payments_this_round: 0,
    standings: [],
    ...overrides,
  }
}

export function makeClusters(overrides = {}) {
  return {
    clusters: [
      {
        root: 'r1',
        addresses: ['bc1qa', 'bc1qb'],
        identity: { identity: 'known', players: ['p1'] },
        names: ['Padaria da Lu'],
        color: '#e0a030',
        balance: 1500000,
      },
      {
        root: 'r2',
        addresses: ['bc1qc'],
        identity: { identity: 'unknown' },
        names: [],
        color: null,
        balance: 60000,
      },
      {
        root: 'r3',
        addresses: ['bc1qd', 'bc1qe'],
        identity: { identity: 'conflict', players: ['p1', 'p2'] },
        names: ['Padaria da Lu', 'Fazenda do Caio'],
        color: '#4caf50',
        balance: 300000,
      },
    ],
    edges: [{ txid: '33'.repeat(32), from: 'r1', to: 'r2', amount: 60000 }],
    ...overrides,
  }
}
