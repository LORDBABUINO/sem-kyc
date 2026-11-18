import { useState } from 'react'
import { api } from '../../services/api.js'
import SectorBadge from '../../components/SectorBadge.jsx'
import { sectorIcon } from '../../data/sectors.js'
import { formatSats } from '../../utils/format.js'
import styles from './Phone.module.css'

// Valor da taxa de cadastro, fixo no roteiro.
export const REGISTRATION_FEE = 10000

export default function Register({ me, onRegistered }) {
  const [error, setError] = useState(null)
  const [busy, setBusy] = useState(false)
  const { player } = me

  const pay = async () => {
    setBusy(true)
    setError(null)
    try {
      await api.register()
      onRegistered()
    } catch (e) {
      setError(e.message)
      setBusy(false)
    }
  }

  return (
    <div className={styles.centered}>
      <div className={styles.badgeIcon} aria-hidden="true" style={{ fontSize: 72 }}>
        {sectorIcon(player.sector)}
      </div>
      <h1 className={styles.title}>Você é {player.name}</h1>
      <SectorBadge sector={player.sector} size="lg" />
      <p className={styles.subtitle}>
        A cooperativa dá as boas-vindas. Pague a taxa de cadastro da vitrine pra abrir as portas.
        É a sua primeira transação.
      </p>
      <div className={styles.card}>
        <span className={styles.muted}>saldo inicial</span>
        <span className={styles.big}>{formatSats(me.balance)}</span>
      </div>
      {error && <p className={styles.error}>{error}</p>}
      <button type="button" className={`${styles.btn} ${styles.btnPrimary}`} onClick={pay} disabled={busy}>
        {busy ? 'pagando…' : `pagar ${formatSats(REGISTRATION_FEE)}`}
      </button>
    </div>
  )
}
