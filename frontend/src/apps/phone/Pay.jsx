import { useEffect, useState } from 'react'
import { api } from '../../services/api.js'
import QrScanner from '../../components/QrScanner.jsx'
import SectorBadge from '../../components/SectorBadge.jsx'
import { formatSats, shortId } from '../../utils/format.js'
import { parseScan } from './scan.js'
import { hasUnlock, spendable, showsOrigin, sumAmount } from './coins.js'
import CoinPicker from './CoinPicker.jsx'
import Overlay from './Overlay.jsx'
import styles from './Phone.module.css'

const POLL_MS = 2000

export default function Pay({ me, initialCode, onClose, onPaid, onCounter }) {
  const [step, setStep] = useState(initialCode ? 'loading' : 'scan')
  const [typed, setTyped] = useState('')
  const [invoice, setInvoice] = useState(null)
  const [selected, setSelected] = useState([])
  const [error, setError] = useState(null)
  const [result, setResult] = useState(null)
  const [cameraOff, setCameraOff] = useState(false)

  const manual = me.player.coin_selection === 'manual'
  const coins = spendable(me.coins)
  const canPayjoin = Boolean(invoice && invoice.payjoin && hasUnlock(me, 'payjoin'))

  const load = async (code) => {
    setStep('loading')
    setError(null)
    try {
      const inv = await api.invoice(code)
      setInvoice(inv)
      setSelected([])
      setStep('invoice')
    } catch (e) {
      setError(e.message)
      setStep('scan')
    }
  }

  useEffect(() => {
    if (initialCode) load(initialCode)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialCode])

  const handleText = (text) => {
    const parsed = parseScan(text, me.counter_address)
    if (parsed.kind === 'code') {
      load(parsed.code)
    } else if (parsed.kind === 'counter') {
      onCounter()
    } else if (parsed.kind === 'empty') {
      setError('Digite o código de 6 caracteres que aparece embaixo do QR.')
    } else {
      setError('Esse QR não traz o código do pedido. Digite o código de 6 caracteres.')
    }
  }

  const pay = async (payjoin) => {
    setStep('paying')
    setError(null)
    try {
      const inputs = manual ? selected : undefined
      const out = await api.payInvoice(invoice.code, { inputs, payjoin })
      if (out.result === 'settled') {
        setResult(out)
        setStep('done')
        onPaid(out)
      } else {
        setStep('awaiting')
      }
    } catch (e) {
      setError(e.message)
      setStep('invoice')
    }
  }

  useEffect(() => {
    if (step !== 'awaiting' || !invoice) return undefined
    const timer = setInterval(async () => {
      try {
        const inv = await api.invoice(invoice.code)
        if (inv.status && inv.status.status === 'paid') {
          setResult({ result: 'settled', txid: inv.status.txid })
          setStep('done')
          onPaid({ result: 'settled', txid: inv.status.txid })
        } else if (inv.status && inv.status.status === 'expired') {
          setError('O pedido expirou antes de o vendedor responder.')
          setStep('invoice')
        }
      } catch {
        /* tenta de novo no próximo ciclo */
      }
    }, POLL_MS)
    return () => clearInterval(timer)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step, invoice])

  const total = manual ? sumAmount(coins.filter((c) => selected.includes(c.outpoint))) : null
  const insufficient = manual && invoice && total < invoice.amount

  return (
    <Overlay title="Pagar" onClose={onClose}>
      {step === 'scan' && (
        <>
          {!cameraOff && <QrScanner onScan={handleText} onError={() => setCameraOff(true)} />}
          <p className={styles.muted}>Aponte pro QR do vendedor ou digite o código de 6 caracteres.</p>
          <form
            className={styles.actions}
            onSubmit={(e) => {
              e.preventDefault()
              handleText(typed)
            }}
          >
            <input
              className={`${styles.input} ${styles.codeInput}`}
              value={typed}
              onChange={(e) => setTyped(e.target.value)}
              placeholder="ABC123"
              aria-label="código do pedido"
              maxLength={6}
              autoComplete="off"
            />
            <button type="submit" className={`${styles.btn} ${styles.btnPrimary}`}>
              buscar pedido
            </button>
          </form>
          {error && <p className={styles.error}>{error}</p>}
        </>
      )}

      {step === 'loading' && <p className={styles.muted}>Buscando o pedido…</p>}

      {(step === 'invoice' || step === 'paying') && invoice && (
        <>
          <div className={`${styles.card} ${styles.cardAccent}`} style={{ '--sector': invoice.seller_sector && invoice.seller_sector.color }}>
            <span className={styles.muted}>{invoice.item}</span>
            <span className={styles.big}>{formatSats(invoice.amount)}</span>
            <div className={styles.rowStart}>
              <span>{invoice.seller_name}</span>
              <SectorBadge sector={invoice.seller_sector} size="sm" />
            </div>
            <span className={styles.muted}>pedido {invoice.code}</span>
          </div>

          {manual && (
            <>
              <p className={styles.sectionTitle}>Com qual moeda?</p>
              <CoinPicker
                coins={coins}
                amount={invoice.amount}
                selected={selected}
                onChange={setSelected}
                showOrigin={showsOrigin(me)}
                exactHint={hasUnlock(me, 'odd_price')}
                disabled={step === 'paying'}
              />
            </>
          )}
          {!manual && (
            <p className={styles.muted}>
              {me.player.coin_selection === 'smallest_covering'
                ? 'A carteira usa a menor moeda que cobre o valor.'
                : 'A carteira escolhe as moedas sozinha.'}
            </p>
          )}

          {error && <p className={styles.error}>{error}</p>}

          <div className={styles.actions}>
            {canPayjoin && (
              <button
                type="button"
                className={`${styles.btn} ${styles.btnPrimary}`}
                onClick={() => pay(true)}
                disabled={step === 'paying' || insufficient}
              >
                pagar junto (Payjoin)
              </button>
            )}
            <button
              type="button"
              className={`${styles.btn} ${canPayjoin ? '' : styles.btnPrimary}`}
              onClick={() => pay(false)}
              disabled={step === 'paying' || insufficient}
            >
              {step === 'paying' ? 'pagando…' : 'pagar'}
            </button>
          </div>
        </>
      )}

      {step === 'awaiting' && (
        <div className={`${styles.card} ${styles.cardWarn}`}>
          <strong>Aguardando o vendedor.</strong>
          <span className={styles.small}>Ele escolhe se contribui uma moeda no Payjoin. Pode levar alguns segundos.</span>
        </div>
      )}

      {step === 'done' && (
        <div className={`${styles.card} ${styles.cardOk}`}>
          <strong>Pago.</strong>
          <span className={styles.small}>
            {invoice ? `${invoice.item}, ${formatSats(invoice.amount)} pra ${invoice.seller_name}.` : ''}
          </span>
          {result && result.txid && <span className={styles.mono}>{shortId(result.txid)}</span>}
          <button type="button" className={`${styles.btn} ${styles.btnPrimary}`} onClick={onClose}>
            voltar
          </button>
        </div>
      )}
    </Overlay>
  )
}
