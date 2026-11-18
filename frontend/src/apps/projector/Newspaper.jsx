import { editionFor } from '../../data/editions.js'
import { formatSats, formatNumber } from '../../utils/format.js'
import styles from './Projector.module.css'

function Footer({ state, round }) {
  if (round <= 1 || !state.reveal) {
    return (
      <div className={styles.paperFooter}>
        <p className={styles.footerHeadline}>
          {round <= 1
            ? `Cadastro concluído: ${formatNumber(state.registered)} estabelecimentos.`
            : 'Sem apuração anterior.'}
        </p>
      </div>
    )
  }
  const reveal = state.reveal
  const autos = reveal.autos || []
  return (
    <div className={styles.paperFooter}>
      <div>
        <p className={styles.footerHeadline}>
          {formatNumber(autos.length)} {autos.length === 1 ? 'estabelecimento autuado' : 'estabelecimentos autuados'} pelo Portaria DOE nº {reveal.round}. ICMS lançado: {formatSats(reveal.assessed)}, quitado: {formatSats(reveal.collected)}.
        </p>
        {reveal.headline_name && (
          <p style={{ margin: '0.6vh 0 0', fontStyle: 'italic' }}>
            Caso da edição: {reveal.headline_name}, {formatNumber(reveal.known && reveal.known.attributed_operations)} de {formatNumber(reveal.known && reveal.known.real_operations)} operações identificadas.
          </p>
        )}
      </div>
      <div className={styles.footerNames} data-testid="footer-names">
        {autos.map((auto) => (
          <div key={auto.id}>
            <span>{auto.name}</span>
            <span>
              {formatSats(auto.amount)} · {auto.paid ? 'quitado' : 'em aberto'}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}

export default function Newspaper({ state, hasTicker }) {
  const round = state.round || (state.phase && state.phase.round) || 1
  const edition = editionFor(round)
  const totals = state.totals || { assessed: 0, collected: 0 }
  return (
    <div className={`${styles.scene} ${styles.paper} ${hasTicker ? styles.withTicker : ''}`}>
      <header className={styles.masthead}>
        <div className={styles.mastheadMeta}>
          <span>Rodada {round}</span>
          <span>Edição nº {round}</span>
          <span>Bloco {formatNumber(state.height)}</span>
        </div>
        <h1 className={styles.mastheadName}>O Rastro</h1>
        <p className={styles.mastheadTag}>Órgão oficial do Departamento de Ordem Econômica da República Democrática de Nova Alvorada.</p>
      </header>

      <div className={styles.paperBody}>
        <div>
          <h2 className={styles.headline}>{edition ? edition.headline : 'Edição extraordinária'}</h2>
          <p className={styles.subheadline}>{edition ? edition.subheadline : ''}</p>
          <p className={styles.lead}>
            O DOE informa que, até o bloco {formatNumber(state.height)}, {formatNumber(state.registered)} estabelecimentos
            constam do cadastro estadual. Desde o início da fiscalização foram lançados {formatSats(totals.assessed)} em
            ICMS, dos quais {formatSats(totals.collected)} já deram entrada no guichê. O DOE lembra que toda
            transação é pública e permanente, e que a leitura da cadeia é feita em tempo real.
          </p>
        </div>
        {edition && (
          <aside className={styles.edital} aria-label={`Portaria DOE nº ${round}`}>
            <h3 className={styles.editalTitle}>Portaria DOE nº {round}</h3>
            <p className={styles.editalText}>{edition.edital}</p>
          </aside>
        )}
      </div>

      <Footer state={state} round={round} />
    </div>
  )
}
