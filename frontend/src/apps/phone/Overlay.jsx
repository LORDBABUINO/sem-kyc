import styles from './Phone.module.css'

export default function Overlay({ title, onClose, children }) {
  return (
    <div className={styles.overlay} role="dialog" aria-label={title}>
      <div className={styles.overlayHead}>
        <button type="button" className={styles.iconBtn} onClick={onClose} aria-label="voltar">
          ‹
        </button>
        <h2 className={styles.h2}>{title}</h2>
      </div>
      <div className={styles.overlayBody}>{children}</div>
    </div>
  )
}
