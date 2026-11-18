import { useCallback, useEffect, useState } from 'react'
import { api } from '../../services/api.js'
import { openSocket } from '../../services/socket.js'
import { storage, ADMIN_KEY } from '../../services/storage.js'
import { sectorName, SECTOR_ORDER } from '../../data/sectors.js'
import { formatSats, formatNumber, formatCountdown } from '../../utils/format.js'
import { useCountdown } from '../../utils/useCountdown.js'
import styles from './Presenter.module.css'

export const CONFIG_FIELDS = [
  { key: 'negotiation_secs', label: 'timer da negociação (s)', fallback: 180 },
  { key: 'icms_percent', label: 'ICMS (%)', fallback: 18 },
  { key: 'penalty_percent', label: 'multa de ofício (%)', fallback: 100 },
  { key: 'late_percent', label: 'multa de mora por rodada (%)', fallback: 20 },
  { key: 'target_revenue', label: 'alvo de receita por vendedor (sats)', fallback: 120000 },
]

const PHASE_LABEL = { lobby: 'cadastro', newspaper: 'jornal', negotiation: 'negociação', reveal: 'apuração', final: 'final' }

function describe(result) {
  if (!result || typeof result !== 'object') return 'ok'
  if (result.phase) {
    const phase = typeof result.phase === 'object' ? result.phase : { phase: result.phase }
    return `fase: ${PHASE_LABEL[phase.phase] || phase.phase}${phase.round ? `, rodada ${phase.round}` : ''}`
  }
  if (result.height != null) return `bloco ${result.height}`
  if (result.txid) return `txid ${result.txid}`
  if (result.players) return `${result.players.length} jogador(es)`
  return 'ok'
}

function Login({ onToken, error }) {
  const [value, setValue] = useState('')
  return (
    <form
      className={styles.login}
      onSubmit={(e) => {
        e.preventDefault()
        if (value.trim()) onToken(value.trim())
      }}
    >
      <h1 className={styles.title}>Painel do apresentador</h1>
      <input
        className={styles.input}
        type="password"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder="token do apresentador"
        aria-label="token do apresentador"
        autoFocus
      />
      {error && <p className={styles.error}>{error}</p>}
      <button type="submit" className={`${styles.bigBtn} ${styles.primary}`}>
        entrar
      </button>
    </form>
  )
}

function CoinJoinCard({ room }) {
  const left = useCountdown(room ? room.deadline : null)
  return (
    <div className={styles.card}>
      <h3 className={styles.cardTitle}>Sala de CoinJoin</h3>
      {room ? (
        <span data-testid="coinjoin">
          {formatNumber(room.entries)} dentro · fecha em {formatCountdown(left)}
        </span>
      ) : (
        <span className={styles.flag}>vazia</span>
      )}
    </div>
  )
}

export default function PresenterApp() {
  const [token, setToken] = useState(() => storage.get(ADMIN_KEY))
  const [room, setRoom] = useState(null)
  const [authError, setAuthError] = useState(null)
  const [feedback, setFeedback] = useState(null)
  const [busy, setBusy] = useState(false)
  const [config, setConfig] = useState(() => Object.fromEntries(CONFIG_FIELDS.map((f) => [f.key, String(f.fallback)])))
  const [bots, setBots] = useState('1')

  const refresh = useCallback(async () => {
    try {
      const next = await api.admin.room()
      setRoom(next)
      if (next && next.config) {
        setConfig((current) => {
          const merged = { ...current }
          for (const field of CONFIG_FIELDS) if (next.config[field.key] != null) merged[field.key] = String(next.config[field.key])
          return merged
        })
      }
    } catch (e) {
      if (e.status === 401 || e.status === 403) {
        storage.remove(ADMIN_KEY)
        setToken(null)
        setAuthError('Token recusado.')
      } else {
        setFeedback({ error: true, text: e.message })
      }
    }
  }, [])

  useEffect(() => {
    if (!token) return undefined
    refresh()
    const socket = openSocket(`/ws/admin?token=${encodeURIComponent(token)}`)
    const off = socket.subscribe(() => refresh())
    return () => {
      off()
      socket.close()
    }
  }, [token, refresh])

  const run = async (label, fn) => {
    setBusy(true)
    setFeedback(null)
    try {
      const out = await fn()
      setFeedback({ text: `${label}: ${describe(out)}` })
      await refresh()
    } catch (e) {
      setFeedback({ error: true, text: `${label}: ${e.message}` })
    } finally {
      setBusy(false)
    }
  }

  const onToken = (value) => {
    storage.set(ADMIN_KEY, value)
    setAuthError(null)
    setToken(value)
  }

  if (!token) return <Login onToken={onToken} error={authError} />

  const state = (room && room.state) || {}
  const phase = state.phase || {}
  const players = (room && room.players) || []
  const autos = state.autos_open || []
  const log = (room && room.log) || []
  const canRelease = phase.phase === 'reveal' && Number(phase.round) >= 8

  const saveConfig = () => {
    const body = {}
    for (const field of CONFIG_FIELDS) {
      const value = Number(config[field.key])
      if (!Number.isNaN(value) && config[field.key] !== '') body[field.key] = value
    }
    return run('Configuração', async () => {
      const out = await api.admin.config(body)
      if (out && typeof out === 'object') {
        setConfig((current) => {
          const merged = { ...current }
          for (const field of CONFIG_FIELDS) if (out[field.key] != null) merged[field.key] = String(out[field.key])
          return merged
        })
      }
      return out
    })
  }

  const bySector = SECTOR_ORDER.map((sector) => ({
    sector,
    list: players.filter((p) => (typeof p.sector === 'string' ? p.sector : p.sector && p.sector.sector) === sector),
  }))
  const others = players.filter((p) => !SECTOR_ORDER.includes(typeof p.sector === 'string' ? p.sector : p.sector && p.sector.sector))

  return (
    <div className={styles.app}>
      <div>
        <h1 className={styles.title}>Painel do apresentador</h1>
        <div className={styles.status}>
          <span>
            fase <strong>{PHASE_LABEL[phase.phase] || phase.phase || '…'}</strong>
          </span>
          <span>
            rodada <strong>{formatNumber(state.round)}</strong>
          </span>
          <span>
            bloco <strong>{formatNumber(state.height)}</strong>
          </span>
          <span>
            pagamentos na rodada <strong>{formatNumber(state.payments_this_round)}</strong>
          </span>
          <span>
            pedidos abertos <strong>{formatNumber(room && room.invoices_open)}</strong>
          </span>
          <span>
            cadastrados <strong>{formatNumber(state.registered)}</strong> de <strong>{formatNumber(state.players)}</strong>
          </span>
        </div>
      </div>

      <div className={styles.buttons}>
        <button
          type="button"
          className={styles.bigBtn}
          disabled={busy}
          onClick={() => {
            if (window.confirm('Abrir cadastro reinicia o jogo e apaga a sala atual. Continuar?')) run('Abrir cadastro', api.admin.reset)
          }}
        >
          Abrir cadastro
        </button>
        <button type="button" className={`${styles.bigBtn} ${styles.primary}`} disabled={busy} onClick={() => run('Avançar edição', api.admin.advance)}>
          Avançar edição
        </button>
        <button type="button" className={`${styles.bigBtn} ${styles.primary}`} disabled={busy} onClick={() => run('Abrir negociação', api.admin.open)}>
          Abrir negociação
        </button>
        <button type="button" className={`${styles.bigBtn} ${styles.primary}`} disabled={busy} onClick={() => run('Fechar e apurar', api.admin.close)}>
          Fechar e apurar
        </button>
        <button type="button" className={styles.bigBtn} disabled={busy} onClick={() => run('Cashback Cidadão', api.admin.dust)}>
          Cashback Cidadão
        </button>
        <button type="button" className={styles.bigBtn} disabled={busy} onClick={() => run('Cadastro das exchanges', api.admin.kyc)}>
          Receber cadastro das exchanges
        </button>
        <button type="button" className={styles.bigBtn} disabled={busy || !canRelease} onClick={() => run('Liberar relatório', api.admin.advance)}>
          Liberar relatório
        </button>
        <button type="button" className={styles.bigBtn} disabled={busy} onClick={() => run('Minerar bloco', api.admin.mine)}>
          Minerar bloco
        </button>
      </div>

      {feedback && (
        <div className={`${styles.feedback} ${feedback.error ? styles.feedbackError : ''}`} role="status">
          {feedback.text}
        </div>
      )}

      <div className={styles.card}>
        <h3 className={styles.cardTitle}>Configuração</h3>
        <div className={styles.config}>
          {CONFIG_FIELDS.map((field) => (
            <label key={field.key} className={styles.field}>
              {field.label}
              <input
                type="number"
                value={config[field.key]}
                onChange={(e) => setConfig((c) => ({ ...c, [field.key]: e.target.value }))}
                aria-label={field.label}
              />
            </label>
          ))}
          <label className={styles.field}>
            bots a adicionar
            <input type="number" min="0" value={bots} onChange={(e) => setBots(e.target.value)} aria-label="bots a adicionar" />
          </label>
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <button type="button" className={styles.smallBtn} disabled={busy} onClick={saveConfig}>
            salvar configuração
          </button>
          <button type="button" className={styles.smallBtn} disabled={busy} onClick={() => run('Modo bots', () => api.admin.bots(Number(bots) || 0))}>
            adicionar bots
          </button>
        </div>
      </div>

      <div className={styles.grid}>
        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Jogadores por setor ({players.length})</h3>
          {[...bySector, { sector: 'outros', list: others }].map(({ sector, list }) =>
            list.length === 0 ? null : (
              <div key={sector} className={styles.sector}>
                <span className={styles.sectorName}>{sector === 'outros' ? 'Outros' : sectorName(sector)}</span>
                {list.map((p) => (
                  <div key={p.id} className={styles.player} data-testid="player">
                    <strong>{p.name}</strong>
                    <span>{formatSats(p.balance)}</span>
                    <span className={`${styles.flag} ${p.registered ? styles.flagOk : styles.flagWarn}`}>
                      {p.registered ? 'cadastrado' : 'sem cadastro'}
                    </span>
                    {p.suspended && <span className={`${styles.flag} ${styles.flagBad}`}>suspenso</span>}
                    <span className={`${styles.flag} ${p.mission_complete ? styles.flagOk : ''}`}>
                      {p.mission_complete ? 'missão completa' : 'missão pendente'}
                    </span>
                  </div>
                ))}
              </div>
            ),
          )}
          {players.length === 0 && <span className={styles.flag}>ninguém ainda</span>}
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div className={styles.card}>
            <h3 className={styles.cardTitle}>Autos em aberto ({autos.length})</h3>
            {autos.length === 0 && <span className={styles.flag}>nenhum</span>}
            {autos.map((auto) => (
              <div key={auto.id} className={styles.auto} data-testid="auto">
                <span>
                  <strong>{auto.name}</strong> · {formatSats(auto.amount)} · nº {auto.id} · rodada {auto.round}
                  {auto.late_rounds > 0 ? ` · ${auto.late_rounds} rodada(s) em atraso` : ''}
                </span>
                <button type="button" className={styles.smallBtn} disabled={busy} onClick={() => run(`Perdoar auto ${auto.id}`, () => api.admin.forgive(auto.id))}>
                  perdoar
                </button>
              </div>
            ))}
          </div>

          <CoinJoinCard room={room && room.coinjoin} />

          <div className={styles.card}>
            <h3 className={styles.cardTitle}>Últimas transações</h3>
            <div className={styles.log}>
              {log.length === 0 && <span className={styles.flag}>nada ainda</span>}
              {log.slice(0, 30).map((entry, index) => (
                <div key={`${entry.txid || index}`} className={styles.logEntry} data-testid="log-entry">
                  <span>{entry.height != null ? `#${entry.height}` : 'fila'}</span>
                  <span>{entry.name || entry.player || ''}</span>
                  <span>{entry.direction === 'in' ? '←' : entry.direction === 'out' ? '→' : '·'}</span>
                  <span>{entry.counterparty || ''}</span>
                  <span>{entry.memo || ''}</span>
                  <strong>{entry.amount != null ? formatSats(entry.amount) : ''}</strong>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
