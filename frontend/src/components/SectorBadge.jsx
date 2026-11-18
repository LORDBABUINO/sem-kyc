import { sectorIcon, sectorName } from '../data/sectors.js'
import styles from './SectorBadge.module.css'

export default function SectorBadge({ sector, size = 'md', showIcon = true }) {
  if (!sector) return null
  const color = typeof sector === 'object' && sector.color ? sector.color : 'var(--muted)'
  return (
    <span className={`${styles.badge} ${styles[size] || ''}`} style={{ '--sector': color }}>
      <span className={styles.dot} aria-hidden="true">
        {showIcon ? sectorIcon(sector) : ''}
      </span>
      <span>{sectorName(sector)}</span>
    </span>
  )
}
