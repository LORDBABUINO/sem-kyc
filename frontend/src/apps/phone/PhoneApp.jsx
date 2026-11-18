import { useCallback, useEffect, useState } from 'react'
import { api } from '../../services/api.js'
import { openSocket } from '../../services/socket.js'
import { storage, SESSION_KEY, TAB_KEY } from '../../services/storage.js'
import Entry from './Entry.jsx'
import Register from './Register.jsx'
import SuspendedBanner from './SuspendedBanner.jsx'
import Pay from './Pay.jsx'
import PayAuto from './PayAuto.jsx'
import Settings from './Settings.jsx'
import Report from './Report.jsx'
import Mission from './tabs/Mission.jsx'
import Wallet from './tabs/Wallet.jsx'
import Sell from './tabs/Sell.jsx'
import History from './tabs/History.jsx'
import Badge from './tabs/Badge.jsx'
import { openAutos } from './coins.js'
import styles from './Phone.module.css'

export const TABS = [
  { key: 'mission', label: 'Missão', icon: '✦' },
  { key: 'wallet', label: 'Carteira', icon: '◎' },
  { key: 'sell', label: 'Vender', icon: '▣' },
  { key: 'history', label: 'Histórico', icon: '≡' },
  { key: 'badge', label: 'Crachá', icon: '▮' },
]

export default function PhoneApp() {
  const [session, setSession] = useState(() => storage.get(SESSION_KEY))
  const [me, setMe] = useState(null)
  const [loadError, setLoadError] = useState(null)
  const [tab, setTabState] = useState(() => {
    const saved = storage.get(TAB_KEY)
    return TABS.some((t) => t.key === saved) ? saved : 'mission'
  })
  const [overlay, setOverlay] = useState(null)
  const [novidades, setNovidades] = useState([])

  const setTab = (key) => {
    setTabState(key)
    storage.set(TAB_KEY, key)
  }

  const refresh = useCallback(async () => {
    try {
      setMe(await api.me())
      setLoadError(null)
    } catch (e) {
      if (e.status === 401 || e.status === 403 || e.status === 404) {
        storage.remove(SESSION_KEY)
        setSession(null)
        setMe(null)
      } else {
        setLoadError(e.message)
      }
    }
  }, [])

  useEffect(() => {
    if (!session) return undefined
    refresh()
    const socket = openSocket(`/ws?session=${encodeURIComponent(session)}`)
    const off = socket.subscribe((message) => {
      if (message.event === 'unlock' && message.unlock) {
        setNovidades((list) => (list.includes(message.unlock) ? list : [...list, message.unlock]))
      }
      refresh()
    })
    return () => {
      off()
      socket.close()
    }
  }, [session, refresh])

  const onJoined = ({ session: token, me: joined }) => {
    storage.set(SESSION_KEY, token)
    setMe(joined)
    setSession(token)
  }

  const openAuto = (auto) => setOverlay({ kind: 'auto', auto })

  if (!session) return <div className={styles.app}><Entry onJoined={onJoined} /></div>
  if (!me) {
    return (
      <div className={styles.app}>
        <div className={styles.centered}>
          <p className={styles.muted}>{loadError || 'Carregando…'}</p>
          {loadError && (
            <button type="button" className={styles.btn} onClick={refresh}>
              tentar de novo
            </button>
          )}
        </div>
      </div>
    )
  }
  if (!me.player.registered) {
    return (
      <div className={styles.app} style={{ '--sector': me.player.sector.color }}>
        <Register me={me} onRegistered={refresh} />
      </div>
    )
  }

  const Current = { mission: Mission, wallet: Wallet, sell: Sell, history: History, badge: Badge }[tab]

  return (
    <div className={styles.app} style={{ '--sector': me.player.sector.color }}>
      <SuspendedBanner me={me} onPay={openAuto} />
      <Current
        me={me}
        onMe={setMe}
        onOpen={setOverlay}
        onTab={setTab}
        novidades={novidades}
        onDismiss={(index) => setNovidades((list) => list.filter((_, i) => i !== index))}
      />

      <nav className={styles.tabs} role="tablist" aria-label="abas">
        {TABS.map((entry) => (
          <button
            key={entry.key}
            type="button"
            role="tab"
            aria-selected={tab === entry.key}
            aria-label={entry.label}
            className={styles.tab}
            onClick={() => setTab(entry.key)}
          >
            <span className={styles.tabIcon} aria-hidden="true">
              {entry.icon}
            </span>
            {entry.label}
          </button>
        ))}
      </nav>

      {overlay && overlay.kind === 'pay' && (
        <Pay
          me={me}
          initialCode={overlay.code}
          onClose={() => setOverlay(null)}
          onPaid={refresh}
          onCounter={() => {
            const auto = openAutos(me)[0]
            if (auto) openAuto(auto)
            else setOverlay(null)
          }}
        />
      )}
      {overlay && overlay.kind === 'auto' && (
        <PayAuto me={me} auto={overlay.auto} onClose={() => setOverlay(null)} onPaid={refresh} />
      )}
      {overlay && overlay.kind === 'settings' && (
        <Settings me={me} highlight={overlay.highlight} onClose={() => setOverlay(null)} onMe={setMe} />
      )}
      {overlay && overlay.kind === 'report' && <Report onClose={() => setOverlay(null)} />}
    </div>
  )
}
