import { useEffect, useRef, useState } from 'react'
import jsQR from 'jsqr'
import styles from './QrScanner.module.css'

export default function QrScanner({ onScan, onError }) {
  const videoRef = useRef(null)
  const [message, setMessage] = useState(null)

  useEffect(() => {
    let stream = null
    let frame = null
    let stopped = false
    const canvas = document.createElement('canvas')
    const schedule = window.requestAnimationFrame ? (fn) => requestAnimationFrame(fn) : (fn) => setTimeout(fn, 50)
    const cancel = window.cancelAnimationFrame ? (id) => cancelAnimationFrame(id) : (id) => clearTimeout(id)

    const fail = (text) => {
      setMessage(text)
      if (onError) onError(text)
    }

    async function start() {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        fail('Câmera indisponível neste navegador. Digite o código.')
        return
      }
      try {
        stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' } })
      } catch {
        fail('Sem acesso à câmera. Digite o código.')
        return
      }
      if (stopped) {
        stream.getTracks().forEach((t) => t.stop())
        return
      }
      const video = videoRef.current
      if (!video) return
      try {
        video.srcObject = stream
        await video.play()
      } catch {
        /* autoplay bloqueado: o loop abaixo segue tentando ler */
      }
      const ctx = canvas.getContext('2d')
      const tick = () => {
        if (stopped) return
        if (ctx && video.readyState >= 2 && video.videoWidth > 0) {
          canvas.width = video.videoWidth
          canvas.height = video.videoHeight
          ctx.drawImage(video, 0, 0, canvas.width, canvas.height)
          const image = ctx.getImageData(0, 0, canvas.width, canvas.height)
          const code = jsQR(image.data, image.width, image.height)
          if (code && code.data) {
            stopped = true
            onScan(code.data)
            return
          }
        }
        frame = schedule(tick)
      }
      frame = schedule(tick)
    }

    start()
    return () => {
      stopped = true
      if (frame) cancel(frame)
      if (stream) stream.getTracks().forEach((t) => t.stop())
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <div className={styles.wrap}>
      <video ref={videoRef} className={styles.video} playsInline muted data-testid="scanner-video" />
      <div className={styles.frame} aria-hidden="true" />
      {message && <p className={styles.message}>{message}</p>}
    </div>
  )
}
