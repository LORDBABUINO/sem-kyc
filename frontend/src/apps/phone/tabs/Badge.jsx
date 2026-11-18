import { sectorIcon, sectorName } from '../../../data/sectors.js'
import styles from '../Phone.module.css'

export default function Badge({ me }) {
  const { player } = me
  return (
    <div className={styles.badgeScreen} style={{ '--sector': player.sector.color }}>
      <div className={styles.badgeIcon} aria-hidden="true">
        {sectorIcon(player.sector)}
      </div>
      <div className={styles.badgeName}>{player.name}</div>
      <div className={styles.badgeSector}>{sectorName(player.sector)}</div>
    </div>
  )
}
