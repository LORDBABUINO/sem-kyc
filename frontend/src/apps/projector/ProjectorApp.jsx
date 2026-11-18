import { useCallback, useEffect, useState } from 'react'
import { api } from '../../services/api.js'
import { openSocket } from '../../services/socket.js'
import Lobby from './Lobby.jsx'
import Newspaper from './Newspaper.jsx'
import Negotiation from './Negotiation.jsx'
import Reveal from './Reveal.jsx'
import Final from './Final.jsx'
import DebtTicker, { hasDebtHistory } from './DebtTicker.jsx'
import styles from './Projector.module.css'

const CLUSTER_EVENTS = new Set(['tx', 'kyc', 'coinjoin', 'reveal', 'phase', 'block'])

export default function ProjectorApp() {
  const [state, setState] = useState(null)
  const [clusters, setClusters] = useState(null)
  const [error, setError] = useState(null)

  const refreshState = useCallback(async () => {
    try {
      setState(await api.state())
      setError(null)
    } catch (e) {
      setError(e.message)
    }
  }, [])

  const refreshClusters = useCallback(async () => {
    try {
      setClusters(await api.clusters())
    } catch {
      /* o grafo fica como está até a próxima transação */
    }
  }, [])

  useEffect(() => {
    refreshState()
    refreshClusters()
    const socket = openSocket('/ws/projector')
    const off = socket.subscribe((message) => {
      refreshState()
      if (CLUSTER_EVENTS.has(message.event)) refreshClusters()
    })
    return () => {
      off()
      socket.close()
    }
  }, [refreshState, refreshClusters])

  if (!state) {
    return (
      <div className={styles.root}>
        {error && <div className={styles.error}>{error}</div>}
      </div>
    )
  }

  const phase = (state.phase && state.phase.phase) || 'lobby'
  const ticker = hasDebtHistory(state)

  return (
    <div className={styles.root} data-phase={phase}>
      {error && <div className={styles.error}>{error}</div>}
      {phase === 'lobby' && <Lobby state={state} />}
      {phase === 'newspaper' && <Newspaper state={state} hasTicker={ticker} />}
      {phase === 'negotiation' && <Negotiation state={state} clusters={clusters} hasTicker={ticker} />}
      {phase === 'reveal' && <Reveal state={state} clusters={clusters} hasTicker={ticker} />}
      {phase === 'final' && <Final state={state} hasTicker={ticker} />}
      <DebtTicker state={state} hideQr={phase === 'reveal'} />
    </div>
  )
}
