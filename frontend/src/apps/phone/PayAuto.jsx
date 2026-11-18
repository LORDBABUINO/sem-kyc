import { useState } from 'react'
import { api } from '../../services/api.js'
import QrScanner from '../../components/QrScanner.jsx'
import { formatSats, shortId } from '../../utils/format.js'
import { parseScan } from './scan.js'
import { hasUnlock, spendable, showsOrigin, sumAmount } from './coins.js'
import CoinPicker from './CoinPicker.jsx'
import Overlay from './Overlay.jsx'
import styles from './Phone.module.css'

export default function PayAuto({ me, auto, onClose, onPaid }) {
  const [selected, setSelected] = useState([])
  const [scanning, setScanning] = useState(false)
  const [error, setError] = useState(null)
  const [busy, setBusy] = useState(false)
  const [txid, setTxid] = useState(null)

  const manual = me.player.coin_selection === 'manual'
  const coins = spendable(me.coins)
  const total = manual ? sumAmount(coins.filter((c) => selected.includes(c.outpoint))) : null
  const insufficient = manual && total < auto.amount

  const pay = async () => {
    setBusy(true)
    setError(null)
    try {
      const out = await api.payAuto(auto.id, manual ? selected : undefined)
      setTxid(out.txid)
      onPaid(out)
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }

  const onScan = (text) => {
    const parsed = parseScan(text, me.counter_address)
    setScanning(false)
    if (parsed.kind === 'counter') pay()
    else setError('Esse QR não é o do guichê do ICMS.')
  }

  return (
    <Overlay title="Guichê do ICMS" onClose={onClose}>
      <div className={`${styles.card}`} style={{ borderColor: 'var(--danger)' }}>
        <span className={styles.muted}>Auto de Infração nº {auto.id}</span>
        <span className={styles.big}>{formatSats(auto.amount)}</span>
        <span className={styles.small}>
          Edital nº {auto.round}
          {auto.late_rounds > 0 ? ` · ${auto.late_rounds} rodada(s) em atraso, multa de mora somada` : ''}
        </span>
        <span className={styles.mono}>{me.counter_address}</span>
      </div>

      {txid ? (
        <div className={`${styles.card} ${styles.cardOk}`}>
          <strong>Auto quitado.</strong>
          <span className={styles.mono}>{shortId(txid)}</span>
          <button type="button" className={`${styles.btn} ${styles.btnPrimary}`} onClick={onClose}>
            voltar
          </button>
        </div>
      ) : (
        <>
          {manual && (
            <>
              <p className={styles.sectionTitle}>Com qual moeda?</p>
              <CoinPicker
                coins={coins}
                amount={auto.amount}
                selected={selected}
                onChange={setSelected}
                showOrigin={showsOrigin(me)}
                exactHint={hasUnlock(me, 'odd_price')}
                disabled={busy}
              />
            </>
          )}
          {scanning && <QrScanner onScan={onScan} onError={() => setScanning(false)} />}
          {error && <p className={styles.error}>{error}</p>}
          <div className={styles.actions}>
            <button
              type="button"
              className={`${styles.btn} ${styles.btnPrimary}`}
              onClick={pay}
              disabled={busy || insufficient}
            >
              {busy ? 'pagando…' : `pagar ${formatSats(auto.amount)}`}
            </button>
            <button type="button" className={styles.btn} onClick={() => setScanning((s) => !s)} disabled={busy}>
              {scanning ? 'fechar câmera' : 'escanear o QR do guichê'}
            </button>
          </div>
        </>
      )}
    </Overlay>
  )
}
