import QrCode from '../../components/QrCode.jsx'
import { formatSats } from '../../utils/format.js'
import styles from './Projector.module.css'

export function hasDebtHistory(state) {
  if (!state) return false
  const open = (state.autos_open || []).length > 0
  const assessed = state.totals && state.totals.assessed > 0
  return open || Boolean(assessed)
}

export default function DebtTicker({ state, hideQr = false }) {
  if (!hasDebtHistory(state)) return null
  const autos = state.autos_open || []
  return (
    <>
      <div className={styles.ticker} data-testid="debt-ticker">
        <div className={styles.tickerLabel}>Dívida Ativa</div>
        <div className={styles.tickerTrack}>
          {autos.length === 0 ? (
            <span className={styles.tickerEmpty}>Todos os autos quitados.</span>
          ) : (
            <div className={styles.tickerInner}>
              {autos.map((auto) => (
                <span key={auto.id}>
                  <strong>{auto.name}</strong> {formatSats(auto.amount)} (Auto nº {auto.id})
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
      {state.counter_address && !hideQr && (
        <div className={styles.cornerQr} data-testid="counter-qr">
          <QrCode value={`bitcoin:${state.counter_address}`} size={96} label="QR do guichê do ICMS" />
          <span>Guichê do DOE</span>
        </div>
      )}
    </>
  )
}
