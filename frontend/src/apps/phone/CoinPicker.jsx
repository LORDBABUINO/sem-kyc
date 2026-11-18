import { formatSats } from '../../utils/format.js'
import { kindLabel, originLabel, sumAmount } from './coins.js'
import styles from './Phone.module.css'

export default function CoinPicker({
  coins,
  amount,
  selected,
  onChange,
  showOrigin = false,
  exactHint = false,
  single = false,
  disabled = false,
}) {
  const chosen = coins.filter((c) => selected.includes(c.outpoint))
  const total = sumAmount(chosen)
  const diff = amount != null ? total - amount : null

  const toggle = (outpoint) => {
    if (single) {
      onChange([outpoint])
      return
    }
    onChange(
      selected.includes(outpoint) ? selected.filter((o) => o !== outpoint) : [...selected, outpoint],
    )
  }

  return (
    <div className={styles.stack}>
      <div className={styles.list}>
        {coins.length === 0 && <div className={styles.item}>Nenhuma moeda disponível.</div>}
        {coins.map((coin) => {
          const exact = exactHint && amount != null && coin.amount === amount
          return (
            <label key={coin.outpoint} className={styles.item}>
              <input
                type={single ? 'radio' : 'checkbox'}
                className={styles.check}
                name={single ? 'coin' : undefined}
                checked={selected.includes(coin.outpoint)}
                onChange={() => toggle(coin.outpoint)}
                disabled={disabled}
                aria-label={`moeda de ${formatSats(coin.amount)}`}
              />
              <div className={`${styles.stack} ${styles.grow}`}>
                <span className={styles.amount}>{formatSats(coin.amount)}</span>
                <span className={`${styles.muted} ${styles.small}`}>
                  {kindLabel(coin.kind)}
                  {showOrigin && coin.origin ? ` · ${originLabel(coin.origin)}` : ''}
                </span>
                {exact && <span className={`${styles.small}`} style={{ color: 'var(--ok)', fontWeight: 700 }}>pague sem troco com esta</span>}
              </div>
            </label>
          )
        })}
      </div>
      {amount != null && !single && (
        <div className={`${styles.row} ${styles.small}`}>
          <span>
            selecionado: <strong>{formatSats(total)}</strong>
          </span>
          {diff >= 0 ? (
            <span>
              troco: <strong>{formatSats(diff)}</strong>
            </span>
          ) : (
            <span style={{ color: 'var(--danger)' }}>faltam {formatSats(-diff)}</span>
          )}
        </div>
      )}
    </div>
  )
}
