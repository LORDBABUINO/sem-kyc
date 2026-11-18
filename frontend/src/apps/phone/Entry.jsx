import { useState } from 'react'
import { api } from '../../services/api.js'
import styles from './Phone.module.css'

export default function Entry({ onJoined }) {
  const [name, setName] = useState('')
  const [error, setError] = useState(null)
  const [busy, setBusy] = useState(false)

  const submit = async (event) => {
    event.preventDefault()
    const trimmed = name.trim()
    if (!trimmed) {
      setError('Dê um nome fantasia pro seu negócio.')
      return
    }
    setBusy(true)
    setError(null)
    try {
      const out = await api.join(trimmed)
      onJoined(out)
    } catch (e) {
      setError(e.message)
    } finally {
      setBusy(false)
    }
  }

  return (
    <form className={styles.centered} onSubmit={submit}>
      <h1 className={styles.title}>O Rastro</h1>
      <p className={styles.subtitle}>
        Você vai abrir um negócio na República Democrática de Nova Alvorada e comprar insumos dos outros, pagando em bitcoin.
        Escolha um nome fantasia.
      </p>
      <input
        className={styles.input}
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Padaria da Lu"
        aria-label="nome fantasia"
        maxLength={40}
        autoFocus
      />
      {error && <p className={styles.error}>{error}</p>}
      <button type="submit" className={`${styles.btn} ${styles.btnPrimary}`} disabled={busy}>
        {busy ? 'entrando…' : 'entrar'}
      </button>
    </form>
  )
}
