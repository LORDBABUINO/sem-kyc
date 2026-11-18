import { useState } from 'react'
import { api } from '../../../services/api.js'
import { formatSats, formatCountdown } from '../../../utils/format.js'
import { useCountdown } from '../../../utils/useCountdown.js'
import { hasUnlock, kindLabel, originLabel, showsOrigin } from '../coins.js'
import styles from '../Phone.module.css'

export const COINJOIN_DENOMINATION = 100000

function CoinJoinRoom({ room }) {
  const left = useCountdown(room ? room.deadline : null)
  if (!room) return null
  return (
    <div className={`${styles.card} ${styles.cardWarn}`} data-testid="coinjoin-room">
      <strong>Sala do CoinJoin</strong>
      <span className={styles.small}>
        {room.entries} dentro · fecha em {formatCountdown(left)} · precisa de 5
      </span>
    </div>
  )
}

export default function Wallet({ me, onMe, onOpen }) {
  const [expanded, setExpanded] = useState(null)
  const [migrating, setMigrating] = useState(false)
  const [marked, setMarked] = useState([])
  const [error, setError] = useState(null)
  const [notice, setNotice] = useState(null)
  const [busy, setBusy] = useState(false)

  const showOrigin = showsOrigin(me)
  const canFreeze = hasUnlock(me, 'freeze')
  const canLightning = hasUnlock(me, 'lightning')
  const canCoinJoin = hasUnlock(me, 'coin_join')
  const canTaproot = hasUnlock(me, 'taproot')

  const run = async (fn, done) => {
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      const out = await fn()
      onMe(await api.me())
      if (done) setNotice(done(out))
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }

  const freeze = (coin) => run(() => api.freeze(coin.outpoint, !coin.frozen))
  const lightning = (coin) => run(() => api.lightning(coin.outpoint), () => 'Saque via Lightning feito. A moeda voltou nova.')
  const coinjoin = (coin) =>
    run(
      () => api.coinjoin(coin.outpoint),
      (out) => `Você entrou na sala: ${out.entries} dentro. A mistura fecha com 5.`,
    )
  const migrate = (together) =>
    run(
      () => api.taproot(marked, together),
      (out) => `Migração enviada: ${(out.txids || []).length} transação(ões).`,
    ).then(() => {
      setMarked([])
      setMigrating(false)
    })

  const toggleMark = (outpoint) =>
    setMarked((list) => (list.includes(outpoint) ? list.filter((o) => o !== outpoint) : [...list, outpoint]))

  return (
    <div className={styles.screen}>
      <div className={styles.row}>
        <div className={styles.stack}>
          <span className={styles.muted}>saldo</span>
          <span className={styles.big}>{formatSats(me.balance)}</span>
        </div>
        <button type="button" className={styles.iconBtn} aria-label="configurações" onClick={() => onOpen({ kind: 'settings' })}>
          ⚙
        </button>
      </div>

      <CoinJoinRoom room={me.coinjoin} />
      {notice && <div className={`${styles.card} ${styles.cardOk}`}>{notice}</div>}
      {error && <p className={styles.error}>{error}</p>}

      {canTaproot && (
        <div className={styles.card}>
          <div className={styles.row}>
            <strong>Migrar pra Taproot</strong>
            <button
              type="button"
              className={`${styles.btn} ${styles.btnSmall}`}
              onClick={() => {
                setMigrating((m) => !m)
                setMarked([])
              }}
            >
              {migrating ? 'cancelar' : 'escolher moedas'}
            </button>
          </div>
          {migrating && marked.length > 1 && (
            <p className={styles.warnText}>Juntar liga as moedas: {marked.length} moedas numa transação só ficam com o mesmo nome.</p>
          )}
          {migrating && marked.length > 0 && (
            <div className={styles.actions}>
              <button type="button" className={`${styles.btn} ${styles.btnPrimary}`} disabled={busy} onClick={() => migrate(false)}>
                {marked.length > 1 ? 'uma transação por moeda' : 'migrar esta moeda'}
              </button>
              {marked.length > 1 && (
                <button type="button" className={styles.btn} disabled={busy} onClick={() => migrate(true)}>
                  juntar numa transação só
                </button>
              )}
            </div>
          )}
        </div>
      )}

      <p className={styles.sectionTitle}>{me.coins.length} moeda(s)</p>
      <div className={styles.list}>
        {me.coins.map((coin) => {
          const open = expanded === coin.outpoint
          const migratable = migrating && coin.kind !== 'taproot' && !coin.frozen
          return (
            <div key={coin.outpoint} className={`${styles.item} ${open ? styles.itemActive : ''}`} style={{ flexDirection: 'column', alignItems: 'stretch' }}>
              <div className={styles.row}>
                {migrating && (
                  <input
                    type="checkbox"
                    className={styles.check}
                    disabled={!migratable}
                    checked={marked.includes(coin.outpoint)}
                    onChange={() => toggleMark(coin.outpoint)}
                    aria-label={`migrar ${formatSats(coin.amount)}`}
                  />
                )}
                <button
                  type="button"
                  className={styles.item}
                  style={{ padding: 0, border: 'none' }}
                  onClick={() => setExpanded(open ? null : coin.outpoint)}
                  aria-expanded={open}
                >
                  <div className={`${styles.stack} ${styles.grow}`}>
                    <span className={styles.amount}>{formatSats(coin.amount)}</span>
                    <span className={`${styles.muted} ${styles.small}`}>
                      {kindLabel(coin.kind)}
                      {showOrigin && coin.origin ? ` · ${originLabel(coin.origin)}` : ''}
                    </span>
                  </div>
                  {coin.frozen && <span className={`${styles.pill} ${styles.pillWarn}`}>congelada</span>}
                </button>
              </div>
              {open && (
                <div className={styles.stack} style={{ marginTop: 10, gap: 8 }}>
                  <span className={styles.mono}>{coin.address}</span>
                  <span className={styles.mono}>{coin.outpoint}</span>
                  <div className={styles.actions}>
                    {canFreeze && (
                      <button type="button" className={`${styles.btn} ${styles.btnSmall}`} disabled={busy} onClick={() => freeze(coin)}>
                        {coin.frozen ? 'descongelar' : 'congelar'}
                      </button>
                    )}
                    {canLightning && coin.origin && coin.origin.kind === 'exchange_withdrawal' && (
                      <button type="button" className={`${styles.btn} ${styles.btnSmall}`} disabled={busy} onClick={() => lightning(coin)}>
                        sacar via Lightning
                      </button>
                    )}
                    {canCoinJoin && !coin.frozen && (
                      <>
                        <button type="button" className={`${styles.btn} ${styles.btnSmall}`} disabled={busy} onClick={() => coinjoin(coin)}>
                          entrar no CoinJoin
                        </button>
                        {coin.amount > COINJOIN_DENOMINATION && (
                          <span className={`${styles.small} ${styles.muted}`}>
                            denominação {formatSats(COINJOIN_DENOMINATION)}: o excedente volta como troco
                          </span>
                        )}
                      </>
                    )}
                    {!canFreeze && !canLightning && !canCoinJoin && !canTaproot && (
                      <span className={`${styles.small} ${styles.muted}`}>Sem ações disponíveis ainda.</span>
                    )}
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
