import { useEffect, useState } from 'react'
import { itemName } from '../../../utils/items.js'
import { api } from '../../../services/api.js'
import QrCode from '../../../components/QrCode.jsx'
import { formatSats } from '../../../utils/format.js'
import { spendable } from '../coins.js'
import CoinPicker from '../CoinPicker.jsx'
import styles from '../Phone.module.css'

export const RECEIVED_FLASH_MS = 2000

function PayjoinRequest({ me, invoice, onMe }) {
  const [selected, setSelected] = useState([])
  const [error, setError] = useState(null)
  const [busy, setBusy] = useState(false)
  const buyer = (invoice.status && (invoice.status.buyer_name || invoice.status.buyer)) || 'Um comprador'

  const answer = async (contribute) => {
    setBusy(true)
    setError(null)
    try {
      await api.answerPayjoin(invoice.code, contribute)
      onMe(await api.me())
    } catch (e) {
      setError(e.message)
      setBusy(false)
    }
  }

  return (
    <div className={`${styles.card} ${styles.cardWarn}`} data-testid="payjoin-request">
      <strong>{buyer} quer pagar junto.</strong>
      <span className={styles.small}>
        {itemName(invoice.item)}, {formatSats(invoice.amount)}. Contribuir uma moeda?
      </span>
      <CoinPicker coins={spendable(me.coins)} selected={selected} onChange={setSelected} single />
      {error && <p className={styles.error}>{error}</p>}
      <div className={styles.actions}>
        <button
          type="button"
          className={`${styles.btn} ${styles.btnPrimary}`}
          disabled={busy || selected.length === 0}
          onClick={() => answer(selected[0])}
        >
          contribuir esta moeda
        </button>
        <button type="button" className={styles.btn} disabled={busy} onClick={() => answer(null)}>
          recusar (sai normal)
        </button>
      </div>
    </div>
  )
}

export default function Sell({ me, onMe }) {
  const [invoice, setInvoice] = useState(null)
  const [flash, setFlash] = useState(null)
  const [error, setError] = useState(null)
  const [busy, setBusy] = useState(false)

  const create = async (item) => {
    setBusy(true)
    setError(null)
    try {
      setInvoice(await api.createInvoice(item))
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }

  useEffect(() => {
    if (!invoice) return undefined
    const current = (me.invoices || []).find((i) => i.code === invoice.code)
    if (!current) return undefined
    const status = current.status && current.status.status
    if (status === 'paid') {
      const entry = (me.history || []).find((h) => h.txid === current.status.txid)
      const who = entry && entry.counterparty ? ` de ${entry.counterparty}` : ''
      setFlash(`recebido${who}, ${formatSats(current.amount)}`)
      setInvoice(null)
    } else if (status === 'expired') {
      setInvoice(null)
      setError('O pedido expirou com o fim da rodada.')
    }
    return undefined
  }, [me.invoices, me.history, invoice])

  useEffect(() => {
    if (!flash) return undefined
    const timer = setTimeout(() => setFlash(null), RECEIVED_FLASH_MS)
    return () => clearTimeout(timer)
  }, [flash])

  const requests = me.payjoin_requests || []

  return (
    <div className={styles.screen}>
      <h1 className={styles.title}>Vender</h1>
      {flash && <div className={styles.toast}>{flash}</div>}

      {requests.map((req) => (
        <PayjoinRequest key={req.code} me={me} invoice={req} onMe={onMe} />
      ))}

      {invoice ? (
        <>
          <div className={styles.qrBox}>
            <QrCode value={invoice.uri} size={300} label="QR do pedido" />
            <span className={styles.big}>{formatSats(invoice.amount)}</span>
            <span>{itemName(invoice.item)}</span>
            <span className={styles.code} aria-label="código do pedido">
              {invoice.code}
            </span>
            <span className={`${styles.muted} ${styles.small}`}>ou digite o código</span>
            {invoice.payjoin && <span className={styles.pill}>Payjoin</span>}
          </div>
          <button type="button" className={styles.btn} onClick={() => setInvoice(null)}>
            voltar pro catálogo
          </button>
        </>
      ) : (
        <>
          <p className={styles.muted}>Toque no item que o comprador pediu.</p>
          {error && <p className={styles.error}>{error}</p>}
          <div className={styles.list}>
            {(me.catalog || []).map((entry) => {
              const closed = !entry.price
              return (
                <button
                  key={entry.name}
                  type="button"
                  className={styles.item}
                  disabled={me.suspended || closed || busy}
                  onClick={() => create(entry.name)}
                >
                  <span className={styles.grow}>{entry.name}</span>
                  <span className={styles.amount}>{closed ? 'fora da negociação' : formatSats(entry.price)}</span>
                </button>
              )
            })}
          </div>
          {me.suspended && <span className={styles.legend}>regularize pra operar</span>}
        </>
      )}
    </div>
  )
}
