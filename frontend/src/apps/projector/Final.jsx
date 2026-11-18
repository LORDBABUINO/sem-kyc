import { FINAL_EDITION } from '../../data/editions.js'
import { formatSats } from '../../utils/format.js'
import styles from './Projector.module.css'

export default function Final({ state, hasTicker }) {
  const top = (state.standings || []).slice(0, 3)
  const order = top.length === 3 ? [top[1], top[0], top[2]] : top
  return (
    <div className={`${styles.scene} ${styles.final} ${hasTicker ? styles.withTicker : ''}`}>
      <div>
        <p className={styles.cardTitle}>Edição final</p>
        <h1 className={styles.headline} style={{ color: 'var(--graph-fg)', fontFamily: 'var(--font-serif)' }}>
          {FINAL_EDITION.headline}
        </h1>
        <p className={styles.subheadline} style={{ color: 'var(--graph-muted)', fontFamily: 'var(--font-serif)' }}>
          {FINAL_EDITION.subheadline}
        </p>
      </div>

      <div className={styles.podium}>
        {order.map((entry) => {
          const place = top.indexOf(entry) + 1
          return (
            <div key={entry.player} className={`${styles.place} ${place === 1 ? styles.first : ''}`} data-testid="standing">
              <span className={styles.placeNumber}>{place}º</span>
              <span className={styles.placeName}>{entry.name}</span>
              <span className={styles.placeBalance}>{formatSats(entry.balance)}</span>
            </div>
          )
        })}
        {top.length === 0 && <p>Placar em apuração.</p>}
      </div>
      <p style={{ margin: 0, color: 'var(--graph-muted)', fontSize: 'clamp(16px, 1.6vw, 28px)' }}>Ninguém aqui ficou parado.</p>

      <ul className={styles.takeaways}>
        {FINAL_EDITION.takeaways.map((text) => (
          <li key={text}>{text}</li>
        ))}
      </ul>
    </div>
  )
}
