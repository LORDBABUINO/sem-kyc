import ClusterGraph from './ClusterGraph.jsx'
import QrCode from '../../components/QrCode.jsx'
import VulnerabilityBadge from '../../components/VulnerabilityBadge.jsx'
import { formatSats, formatNumber } from '../../utils/format.js'
import { findRootFor } from './graphModel.js'
import styles from './Projector.module.css'

function findingType(finding) {
  if (!finding) return null
  return typeof finding === 'string' ? finding : finding.type || null
}

export default function Reveal({ state, clusters, hasTicker }) {
  const reveal = state.reveal
  const headline = reveal && reveal.headline
  const focusRoot = headline
    ? findRootFor(clusters, { playerId: headline.player, name: reveal.headline_name })
    : null
  const known = (reveal && reveal.known) || {}
  const autos = (reveal && reveal.autos) || []
  const type = findingType(headline && headline.finding)
  const severity = headline && headline.finding && typeof headline.finding === 'object' ? headline.finding.severity : null

  return (
    <div className={`${styles.scene} ${hasTicker ? styles.withTicker : ''}`}>
      <div className={styles.revealGrid}>
        <div className={styles.revealGraph}>
          <ClusterGraph data={clusters} focusRoot={focusRoot} />
        </div>

        <div className={styles.card} data-testid="state-knows">
          <h3 className={styles.cardTitle}>O que o DOE sabe</h3>
          {reveal && reveal.headline_name ? (
            <>
              <p className={styles.cardName}>{reveal.headline_name}</p>
              {type && <VulnerabilityBadge type={type} severity={severity} />}
              <div className={styles.kv}>
                <span>operações identificadas</span>
                <strong>
                  {formatNumber(known.attributed_operations)} de {formatNumber(known.real_operations)}
                </strong>
              </div>
              <div className={styles.kv}>
                <span>endereços ligados</span>
                <strong>{formatNumber(known.linked_addresses)}</strong>
              </div>
              <div className={styles.kv}>
                <span>contrapartes identificadas</span>
                <strong>{(known.counterparties || []).length ? known.counterparties.join(', ') : 'nenhuma'}</strong>
              </div>
              <div className={styles.kv}>
                <span>ICMS lançado na rodada</span>
                <strong>{formatSats(reveal.assessed)}</strong>
              </div>
            </>
          ) : (
            <p className={styles.cardName} style={{ fontSize: 'clamp(20px, 2.2vw, 40px)' }}>
              Nenhum caso identificado nesta rodada.
            </p>
          )}
        </div>

        <div className={styles.autosRow}>
          <div className={styles.card}>
            <h3 className={styles.cardTitle}>
              Autuados da rodada {reveal ? reveal.round : state.round} · Portaria DOE nº {reveal ? reveal.round : state.round}
            </h3>
            {autos.length === 0 ? (
              <p style={{ margin: 0 }}>Nenhum auto de infração lavrado.</p>
            ) : (
              <div className={styles.autosList}>
                {autos.map((auto) => (
                  <div key={auto.id} className={styles.autoItem} data-testid="reveal-auto">
                    <span>
                      {auto.name}
                      <small>Auto nº {auto.id}</small>
                    </span>
                    <strong>{formatSats(auto.amount)}</strong>
                  </div>
                ))}
              </div>
            )}
            <div className={styles.totals}>
              <span>
                lançado na rodada: <strong>{formatSats(reveal ? reveal.assessed : 0)}</strong>
              </span>
              <span>
                quitado: <strong>{formatSats(reveal ? reveal.collected : 0)}</strong>
              </span>
              <span>
                pagamentos: <strong>{formatNumber(reveal ? reveal.payments : 0)}</strong>
              </span>
            </div>
          </div>
          {state.counter_address && (
            <div className={styles.bigQr}>
              <QrCode value={`bitcoin:${state.counter_address}`} size={Math.min(260, Math.round(window.innerHeight * 0.28))} label="QR grande do guichê" />
              <span>Guichê do DOE</span>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
