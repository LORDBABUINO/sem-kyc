import { useState } from 'react'
import { api } from '../../services/api.js'
import { hasUnlock } from './coins.js'
import Overlay from './Overlay.jsx'
import styles from './Phone.module.css'

export const SETTING_GROUPS = [
  {
    key: 'receive_mode',
    title: 'Modo de recebimento',
    options: [
      { value: 'storefront', label: 'vitrine (endereço fixo)' },
      { value: 'fresh', label: 'endereço novo a cada venda', unlock: 'fresh_address' },
      { value: 'payjoin', label: 'Payjoin', unlock: 'payjoin' },
      { value: 'silent_payments', label: 'Silent Payments (sp1)', unlock: 'silent_payments' },
    ],
  },
  {
    key: 'price_style',
    title: 'Estilo de preço',
    options: [
      { value: 'round', label: 'redondo' },
      { value: 'odd', label: 'quebrado', unlock: 'odd_price' },
    ],
  },
  {
    key: 'coin_selection',
    title: 'Seleção de moeda',
    options: [
      { value: 'automatic', label: 'automática (junta até cobrir)' },
      { value: 'smallest_covering', label: 'a menor moeda que cobre', unlock: 'coin_control' },
      { value: 'manual', label: 'manual', unlock: 'coin_control' },
    ],
  },
]

export default function Settings({ me, highlight, onClose, onMe }) {
  const [error, setError] = useState(null)
  const [busy, setBusy] = useState(false)
  const [sp, setSp] = useState(null)

  const change = async (key, value) => {
    setBusy(true)
    setError(null)
    try {
      onMe(await api.settings({ [key]: value }))
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }

  const registerSilent = async () => {
    setBusy(true)
    setError(null)
    try {
      const out = await api.silentStorefront()
      setSp(out.sp)
      onMe(await api.me())
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }

  return (
    <Overlay title="Configurações" onClose={onClose}>
      {SETTING_GROUPS.map((group) => (
        <section key={group.key} aria-label={group.title}>
          <p className={styles.sectionTitle} style={highlight === group.key ? { color: 'var(--sector)' } : undefined}>
            {group.title}
          </p>
          <div className={styles.list} role="radiogroup" aria-label={group.title}>
            {group.options.map((opt) => {
              const locked = Boolean(opt.unlock) && !hasUnlock(me, opt.unlock)
              return (
                <label key={opt.value} className={`${styles.option} ${locked ? styles.optionDisabled : ''}`}>
                  <input
                    type="radio"
                    className={styles.check}
                    name={group.key}
                    value={opt.value}
                    aria-label={opt.label}
                    checked={me.player[group.key] === opt.value}
                    disabled={locked || busy}
                    onChange={() => change(group.key, opt.value)}
                  />
                  <span className={styles.grow}>{opt.label}</span>
                  {locked && <span className={styles.pill}>ainda não disponível</span>}
                </label>
              )
            })}
          </div>
        </section>
      ))}

      {hasUnlock(me, 'silent_payments') && (
        <section aria-label="Vitrine">
          <p className={styles.sectionTitle}>Vitrine cadastrada</p>
          <div className={styles.card}>
            {me.player.storefront_is_silent || sp ? (
              <>
                <span>Vitrine recadastrada como Silent Payments.</span>
                <span className={styles.mono}>{sp || me.player.silent_address}</span>
              </>
            ) : (
              <>
                <span className={styles.small}>
                  O DOE pede um endereço fixo. Dê a ele um sp1: cada pagamento cai num lugar diferente.
                </span>
                <button type="button" className={`${styles.btn} ${styles.btnPrimary}`} onClick={registerSilent} disabled={busy}>
                  recadastrar vitrine como sp1
                </button>
              </>
            )}
          </div>
        </section>
      )}

      {error && <p className={styles.error}>{error}</p>}
    </Overlay>
  )
}
