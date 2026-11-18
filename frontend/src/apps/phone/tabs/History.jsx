import { useState } from 'react'
import { formatSats } from '../../../utils/format.js'
import styles from '../Phone.module.css'

const ARROWS = { in: '↓', out: '↑', internal: '↻' }
const DIRECTION = { in: 'entrada', out: 'saída', internal: 'interna' }

export default function History({ me }) {
  const [open, setOpen] = useState(null)
  const entries = me.history || []

  return (
    <div className={styles.screen}>
      <h1 className={styles.title}>Histórico</h1>
      {entries.length === 0 && <div className={styles.card}>Nenhuma transação ainda.</div>}
      <div className={styles.list}>
        {entries.map((entry) => {
          const expanded = open === entry.txid
          const cls = entry.direction === 'in' ? styles.arrowIn : entry.direction === 'out' ? styles.arrowOut : ''
          return (
            <div key={entry.txid} className={`${styles.item} ${expanded ? styles.itemActive : ''}`} style={{ flexDirection: 'column', alignItems: 'stretch' }}>
              <button
                type="button"
                className={styles.item}
                style={{ padding: 0, border: 'none' }}
                onClick={() => setOpen(expanded ? null : entry.txid)}
                aria-expanded={expanded}
              >
                <span className={`${styles.arrow} ${cls}`} aria-label={DIRECTION[entry.direction] || entry.direction}>
                  {ARROWS[entry.direction] || '·'}
                </span>
                <div className={`${styles.stack} ${styles.grow}`}>
                  <span className={styles.ellipsis}>{entry.counterparty || 'desconhecido'}</span>
                  <span className={`${styles.muted} ${styles.small} ${styles.ellipsis}`}>{entry.memo}</span>
                </div>
                <div className={styles.stack} style={{ alignItems: 'flex-end' }}>
                  <span className={styles.amount}>
                    {entry.direction === 'out' ? '-' : entry.direction === 'in' ? '+' : ''}
                    {formatSats(entry.amount)}
                  </span>
                  <span className={`${styles.muted} ${styles.small}`}>
                    {entry.height != null ? `bloco ${entry.height}` : 'na fila'}
                  </span>
                </div>
              </button>
              {expanded && (
                <div className={styles.stack} style={{ marginTop: 10 }}>
                  <span className={`${styles.muted} ${styles.small}`}>txid</span>
                  <span className={styles.mono} data-testid="txid">
                    {entry.txid}
                  </span>
                  <span className={`${styles.muted} ${styles.small}`}>rodada {entry.round}</span>
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
