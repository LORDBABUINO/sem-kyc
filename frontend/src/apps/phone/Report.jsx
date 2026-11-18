import { useEffect, useState } from 'react'
import { api } from '../../services/api.js'
import VulnerabilityBadge from '../../components/VulnerabilityBadge.jsx'
import Overlay from './Overlay.jsx'
import styles from './Phone.module.css'

function asText(value) {
  if (value == null) return ''
  if (typeof value === 'string') return value
  if (typeof value === 'object' && (value.message || value.description || value.text)) {
    return value.message || value.description || value.text
  }
  return JSON.stringify(value, null, 2)
}

export default function Report({ onClose }) {
  const [report, setReport] = useState(null)
  const [error, setError] = useState(null)

  useEffect(() => {
    api.report().then(setReport, (e) => setError(e.message))
  }, [])

  const findings = (report && report.findings) || []
  const warnings = (report && report.warnings) || []

  return (
    <Overlay title="Relatório" onClose={onClose}>
      {error && <p className={styles.error}>{error}</p>}
      {!report && !error && <p className={styles.muted}>Carregando o dossiê…</p>}
      {report && (
        <>
          {report.summary && <p className={styles.subtitle}>{asText(report.summary)}</p>}
          {report.stats && typeof report.stats === 'object' && (
            <div className={styles.card}>
              {Object.entries(report.stats).map(([key, value]) => (
                <div key={key} className={`${styles.row} ${styles.small}`}>
                  <span className={styles.muted}>{key.replace(/_/g, ' ')}</span>
                  <strong>{asText(value)}</strong>
                </div>
              ))}
            </div>
          )}
          <p className={styles.sectionTitle}>
            {findings.length === 0 ? 'Nenhum achado' : `${findings.length} achado(s)`}
          </p>
          {findings.map((finding, index) => (
            <article key={`${finding.type}-${index}`} className={styles.card} data-testid="finding">
              <VulnerabilityBadge type={finding.type} severity={finding.severity} />
              <p style={{ margin: 0 }}>{asText(finding.description)}</p>
              {finding.details != null && (
                <details>
                  <summary className={styles.small}>evidência</summary>
                  <pre className={styles.mono} style={{ whiteSpace: 'pre-wrap', margin: '8px 0 0' }}>
                    {asText(finding.details)}
                  </pre>
                </details>
              )}
              {finding.correction && (
                <div className={`${styles.card} ${styles.cardOk}`} style={{ padding: '10px 12px' }}>
                  <strong className={styles.small}>Correção</strong>
                  <span className={styles.small}>{asText(finding.correction)}</span>
                </div>
              )}
            </article>
          ))}
          {warnings.length > 0 && (
            <>
              <p className={styles.sectionTitle}>Avisos</p>
              {warnings.map((warning, index) => (
                <div key={index} className={`${styles.card} ${styles.cardWarn}`}>
                  {warning && warning.type && <VulnerabilityBadge type={warning.type} severity={warning.severity} />}
                  <span className={styles.small}>{asText(warning)}</span>
                </div>
              ))}
            </>
          )}
        </>
      )}
    </Overlay>
  )
}
