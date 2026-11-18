import { useMemo } from 'react'
import QRCode from 'qrcode'
import styles from './QrCode.module.css'

export default function QrCode({ value, size = 240, label, className }) {
  const { n, path } = useMemo(() => {
    if (!value) return { n: 0, path: '' }
    const qr = QRCode.create(String(value), { errorCorrectionLevel: 'M' })
    const count = qr.modules.size
    const data = qr.modules.data
    let d = ''
    for (let y = 0; y < count; y += 1) {
      for (let x = 0; x < count; x += 1) {
        if (data[y * count + x]) d += `M${x} ${y}h1v1h-1z`
      }
    }
    return { n: count, path: d }
  }, [value])

  if (!n) return null
  const margin = 2
  return (
    <svg
      role="img"
      aria-label={label || value}
      data-value={value}
      className={[styles.qr, className].filter(Boolean).join(' ')}
      viewBox={`${-margin} ${-margin} ${n + margin * 2} ${n + margin * 2}`}
      width={size}
      height={size}
      shapeRendering="crispEdges"
    >
      <rect x={-margin} y={-margin} width={n + margin * 2} height={n + margin * 2} fill="#fff" />
      <path d={path} fill="#000" />
    </svg>
  )
}
