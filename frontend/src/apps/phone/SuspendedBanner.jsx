import { formatSats } from '../../utils/format.js'
import { openAutos } from './coins.js'
import styles from './Phone.module.css'

export default function SuspendedBanner({ me, onPay }) {
  if (!me.suspended) return null
  const autos = openAutos(me)
  return (
    <div className={styles.banner} role="alert">
      <div className={styles.bannerTitle}>Inscrição estadual suspensa.</div>
      {autos.length === 0 && (
        <div className={styles.bannerLine}>
          <span>ICMS devido {formatSats(me.debt)}.</span>
        </div>
      )}
      {autos.map((auto) => (
        <div key={auto.id} className={styles.bannerLine}>
          <span>
            Auto de Infração nº {auto.id}: ICMS devido {formatSats(auto.amount)}.
          </span>
          <button type="button" className={styles.bannerBtn} onClick={() => onPay(auto)}>
            pagar no guichê
          </button>
        </div>
      ))}
    </div>
  )
}
