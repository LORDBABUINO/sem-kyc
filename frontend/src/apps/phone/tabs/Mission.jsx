import SectorBadge from '../../../components/SectorBadge.jsx'
import { itemName } from '../../../utils/items.js'
import { formatSats, formatCountdown } from '../../../utils/format.js'
import { useCountdown } from '../../../utils/useCountdown.js'
import { unlockInfo } from '../../../data/unlocks.js'
import styles from '../Phone.module.css'

const STATUS = {
  pending: { label: 'a comprar', cls: '' },
  paid: { label: 'pago', cls: 'pillOk' },
  expired: { label: 'expirado', cls: 'pillDanger' },
}

function PhaseLine({ me }) {
  const phase = me.phase || {}
  const left = useCountdown(phase.phase === 'negotiation' ? phase.deadline : null)
  if (phase.phase === 'negotiation') {
    return (
      <div className={styles.row}>
        <span className={styles.muted}>negociação aberta</span>
        <span className={styles.big} aria-label="tempo restante">
          {formatCountdown(left)}
        </span>
      </div>
    )
  }
  const text = {
    lobby: 'Aguardando a abertura.',
    newspaper: 'O jornal está no projetor. A negociação abre em seguida.',
    reveal: 'Apuração da rodada no projetor.',
    final: 'Edição final.',
  }[phase.phase]
  return <span className={styles.muted}>{text || ''}</span>
}

export default function Mission({ me, novidades, onDismiss, onOpen, onTab }) {
  const mission = me.mission

  const go = (unlock) => {
    const info = unlockInfo(unlock)
    if (!info) return
    if (info.target === 'settings') onOpen({ kind: 'settings', highlight: info.setting })
    else if (info.target === 'report') onOpen({ kind: 'report' })
    else onTab('wallet')
  }

  return (
    <div className={styles.screen}>
      <div className={styles.row}>
        <h1 className={styles.title}>Rodada {me.round || 0}</h1>
        <span className={styles.muted}>bloco {me.height}</span>
      </div>
      <div className={styles.card}>
        <PhaseLine me={me} />
      </div>

      {novidades.map((unlock, index) => {
        const info = unlockInfo(unlock)
        if (!info) return null
        return (
          <div key={`${unlock}-${index}`} className={`${styles.card} ${styles.cardAccent}`} data-testid="novidade">
            <strong>Novidade na carteira</strong>
            <span>{info.sentence}</span>
            <div className={styles.row}>
              <button type="button" className={`${styles.btn} ${styles.btnSmall} ${styles.btnPrimary}`} onClick={() => go(unlock)}>
                {info.button}
              </button>
              <button type="button" className={`${styles.btn} ${styles.btnSmall}`} onClick={() => onDismiss(index)}>
                ok
              </button>
            </div>
          </div>
        )
      })}

      {me.report_available && (
        <button type="button" className={`${styles.btn} ${styles.btnPrimary}`} onClick={() => onOpen({ kind: 'report' })}>
          Relatório
        </button>
      )}

      {!mission && <div className={styles.card}>Aguardando a mensagem da cooperativa.</div>}

      {mission && (
        <>
          <div className={styles.card}>
            <span className={styles.sectionTitle} style={{ margin: 0 }}>
              Cooperativa
            </span>
            <p style={{ margin: 0, lineHeight: 1.45 }}>{mission.text}</p>
          </div>

          <div className={styles.list}>
            {mission.items.map((entry) => {
              const status = STATUS[entry.status.status] || STATUS.pending
              return (
                <div key={itemName(entry.item)} className={styles.item} data-testid="mission-item">
                  <div className={`${styles.stack} ${styles.grow}`}>
                    <strong>{itemName(entry.item)}</strong>
                    <SectorBadge sector={entry.supplier} size="sm" />
                  </div>
                  <div className={styles.stack} style={{ alignItems: 'flex-end' }}>
                    <span className={styles.amount}>{formatSats(entry.price)}</span>
                    <span className={`${styles.pill} ${styles[status.cls] || ''}`}>{status.label}</span>
                  </div>
                </div>
              )
            })}
          </div>

          {mission.complete && (
            <div className={`${styles.card} ${mission.bonus ? styles.cardOk : ''}`}>
              bônus da cooperativa: {mission.bonus ? 'chegou' : 'a caminho'}
            </div>
          )}
        </>
      )}

      <div className={styles.actions}>
        <button
          type="button"
          className={`${styles.btn} ${styles.btnPrimary}`}
          disabled={me.suspended}
          onClick={() => onOpen({ kind: 'pay' })}
        >
          escanear pra pagar
        </button>
        <button type="button" className={styles.btn} disabled={me.suspended} onClick={() => onOpen({ kind: 'pay', typing: true })}>
          digitar código
        </button>
        {me.suspended && <span className={styles.legend}>regularize pra operar</span>}
      </div>
    </div>
  )
}
