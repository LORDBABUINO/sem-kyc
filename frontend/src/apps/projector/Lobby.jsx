import QrCode from '../../components/QrCode.jsx'
import styles from './Projector.module.css'

export default function Lobby({ state }) {
  return (
    <div className={`${styles.scene} ${styles.lobby}`}>
      <h1 className={styles.lobbyTitle}>Sem KYC</h1>
      <div className={styles.lobbyQr}>
        <QrCode value={state.join_url} size={Math.min(520, Math.round(window.innerHeight * 0.5))} label="QR de entrada" />
      </div>
      <p style={{ margin: 0, fontSize: 'clamp(18px, 2vw, 32px)', color: 'var(--graph-muted)' }}>{state.join_url}</p>
      <div className={styles.lobbyStats}>
        <div>
          <strong>{state.registered}</strong>
          estabelecimentos cadastrados
        </div>
        <div>
          <strong>{state.players}</strong>
          pessoas na sala
        </div>
        <div>
          <strong>{state.height}</strong>
          bloco atual
        </div>
      </div>
    </div>
  )
}
