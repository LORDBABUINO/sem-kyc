import { storage, SESSION_KEY, ADMIN_KEY } from './storage.js'

export async function request(method, path, body, headers = {}) {
  const init = { method, headers: { ...headers } }
  if (body !== undefined) {
    init.headers['Content-Type'] = 'application/json'
    init.body = JSON.stringify(body)
  }
  const res = await fetch(path, init)
  let data = null
  try {
    data = await res.json()
  } catch {
    data = null
  }
  if (!res.ok) {
    const err = new Error((data && data.error) || `Erro ${res.status}`)
    err.status = res.status
    throw err
  }
  return data
}

function sessionHeaders() {
  const token = storage.get(SESSION_KEY)
  return token ? { 'X-Session': token } : {}
}

function adminHeaders() {
  return { 'X-Admin': storage.get(ADMIN_KEY) || '' }
}

const player = (method, path, body) => request(method, path, body, sessionHeaders())
const admin = (method, path, body) => request(method, path, body, adminHeaders())

export const api = {
  join: (name) => request('POST', '/api/join', { name }),
  me: () => player('GET', '/api/me'),
  register: () => player('POST', '/api/register'),
  settings: (patch) => player('POST', '/api/settings', patch),
  freeze: (outpoint, frozen) => player('POST', '/api/coins/freeze', { outpoint, frozen }),
  createInvoice: (item) => player('POST', '/api/invoices', { item }),
  invoice: (code) => player('GET', `/api/invoices/${encodeURIComponent(code)}`),
  payInvoice: (code, { inputs, payjoin } = {}) => {
    const body = {}
    if (inputs) body.inputs = inputs
    if (payjoin) body.payjoin = true
    return player('POST', `/api/invoices/${encodeURIComponent(code)}/pay`, body)
  },
  answerPayjoin: (code, contribute) =>
    player('POST', `/api/invoices/${encodeURIComponent(code)}/payjoin`, { contribute }),
  lightning: (outpoint) => player('POST', '/api/lightning', { outpoint }),
  coinjoin: (outpoint) => player('POST', '/api/coinjoin', { outpoint }),
  taproot: (outpoints, together) => player('POST', '/api/taproot', { outpoints, together }),
  silentStorefront: () => player('POST', '/api/storefront/silent'),
  payAuto: (id, inputs) => player('POST', `/api/autos/${id}/pay`, inputs ? { inputs } : {}),
  report: () => player('GET', '/api/report'),

  state: () => request('GET', '/api/state'),
  clusters: () => request('GET', '/api/clusters'),

  admin: {
    advance: () => admin('POST', '/api/admin/advance'),
    open: () => admin('POST', '/api/admin/open'),
    close: () => admin('POST', '/api/admin/close'),
    dust: () => admin('POST', '/api/admin/dust'),
    kyc: () => admin('POST', '/api/admin/kyc'),
    mine: () => admin('POST', '/api/admin/mine'),
    forgive: (id) => admin('POST', `/api/admin/autos/${id}/forgive`),
    bots: (count) => admin('POST', '/api/admin/bots', { count }),
    reset: () => admin('POST', '/api/admin/reset'),
    room: () => admin('GET', '/api/admin/room'),
    config: (patch) => admin('POST', '/api/admin/config', patch),
  },
}
