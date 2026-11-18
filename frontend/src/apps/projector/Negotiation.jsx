import ClusterGraph from './ClusterGraph.jsx'
import { formatCountdown, formatNumber } from '../../utils/format.js'
import { useCountdown } from '../../utils/useCountdown.js'
import styles from './Projector.module.css'

export default function Negotiation({ state, clusters, hasTicker }) {
  const deadline = state.phase && state.phase.deadline
  const left = useCountdown(deadline)
  return (
    <div className={`${styles.scene} ${hasTicker ? styles.withTicker : ''}`}>
      <div className={styles.topbar}>
        <div className={left != null && left <= 30 ? styles.timerLow : ''}>
          negociação, rodada {state.round}
          <strong aria-label="tempo restante">{formatCountdown(left)}</strong>
        </div>
        <div style={{ textAlign: 'center' }}>
          pagamentos na rodada
          <strong>{formatNumber(state.payments_this_round)}</strong>
        </div>
        <div style={{ textAlign: 'right' }}>
          bloco
          <strong>{formatNumber(state.height)}</strong>
        </div>
      </div>
      <ClusterGraph data={clusters} />
    </div>
  )
}
