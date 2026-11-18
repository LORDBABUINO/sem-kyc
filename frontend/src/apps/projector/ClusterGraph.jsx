import { useEffect, useRef } from 'react'
import { forceSimulation, forceManyBody, forceLink, forceCollide, forceRadial, forceCenter } from 'd3-force'
import { reconcile, ghostPosition, fitCamera, PULSE_MS } from './graphModel.js'
import styles from './Projector.module.css'

const MAX_DOTS = 24

function drawNode(ctx, node, x, y, r, alpha = 1) {
  ctx.save()
  ctx.globalAlpha = alpha
  ctx.beginPath()
  ctx.arc(x, y, r, 0, Math.PI * 2)
  ctx.closePath()
  if (node.colors.length > 1) {
    ctx.save()
    ctx.clip()
    const step = Math.max(6, r / 4)
    for (let i = -r * 2; i < r * 2; i += step) {
      ctx.fillStyle = node.colors[Math.round(i / step) % 2 === 0 ? 0 : 1]
      ctx.beginPath()
      ctx.moveTo(x + i, y - r)
      ctx.lineTo(x + i + step, y - r)
      ctx.lineTo(x + i + step - r * 2, y + r)
      ctx.lineTo(x + i - r * 2, y + r)
      ctx.closePath()
      ctx.fill()
    }
    ctx.restore()
    ctx.beginPath()
    ctx.arc(x, y, r, 0, Math.PI * 2)
    ctx.closePath()
  } else {
    ctx.fillStyle = node.colors[0]
    ctx.fill()
  }
  ctx.lineWidth = node.known ? 3 : 1.5
  ctx.strokeStyle = node.known ? 'rgba(255,255,255,0.9)' : 'rgba(255,255,255,0.35)'
  ctx.stroke()

  const dots = Math.min(MAX_DOTS, (node.addresses || []).length)
  ctx.fillStyle = 'rgba(0,0,0,0.45)'
  for (let i = 0; i < dots; i += 1) {
    const angle = i * 2.399963
    const dist = (r - 6) * Math.sqrt((i + 0.5) / dots)
    ctx.beginPath()
    ctx.arc(x + Math.cos(angle) * dist, y + Math.sin(angle) * dist, 2, 0, Math.PI * 2)
    ctx.fill()
  }
  ctx.restore()
}

export default function ClusterGraph({ data, focusRoot = null, className }) {
  const canvasRef = useRef(null)
  const stateRef = useRef({ nodes: [], links: [], ghosts: [], seenEdges: new Map(), camera: null, simulation: null, focusRoot })

  useEffect(() => {
    stateRef.current.focusRoot = focusRoot
  }, [focusRoot])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return undefined
    const ctx = canvas.getContext ? canvas.getContext('2d') : null
    const st = stateRef.current
    const size = () => {
      const parent = canvas.parentElement
      return { w: (parent && parent.clientWidth) || 1280, h: (parent && parent.clientHeight) || 720 }
    }
    let { w, h } = size()
    const resize = () => {
      ;({ w, h } = size())
      const dpr = window.devicePixelRatio || 1
      canvas.width = Math.round(w * dpr)
      canvas.height = Math.round(h * dpr)
      canvas.style.width = `${w}px`
      canvas.style.height = `${h}px`
    }
    resize()
    window.addEventListener('resize', resize)

    const simulation = forceSimulation([])
      .force('charge', forceManyBody().strength(-140))
      .force('link', forceLink([]).id((d) => d.root).distance(110).strength(0.25))
      .force('collide', forceCollide((d) => d.r + 8))
      .force('radial', forceRadial((d) => (d.known ? 0 : Math.min(w, h) * 0.4), w / 2, h / 2).strength((d) => (d.known ? 0.04 : 0.09)))
      .force('center', forceCenter(w / 2, h / 2).strength(0.02))
      .alphaDecay(0.028)
    st.simulation = simulation

    let frame = null
    const draw = () => {
      frame = requestAnimationFrame(draw)
      if (!ctx) return
      const current = size()
      if (current.w !== w || current.h !== h) resize()
      const now = Date.now()
      const dpr = window.devicePixelRatio || 1
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
      ctx.clearRect(0, 0, w, h)

      const target = fitCamera(st.nodes, w, h, st.focusRoot)
      if (!st.camera) st.camera = { ...target }
      st.camera.x += (target.x - st.camera.x) * 0.08
      st.camera.y += (target.y - st.camera.y) * 0.08
      st.camera.k += (target.k - st.camera.k) * 0.08
      const cam = st.camera
      ctx.translate(w / 2, h / 2)
      ctx.scale(cam.k, cam.k)
      ctx.translate(-cam.x, -cam.y)

      for (const link of st.links) {
        const s = link.source
        const t = link.target
        if (!s || !t || typeof s !== 'object') continue
        const age = now - link.addedAt
        const pulse = age < PULSE_MS ? 1 - age / PULSE_MS : 0
        ctx.beginPath()
        ctx.moveTo(s.x, s.y)
        ctx.lineTo(t.x, t.y)
        ctx.lineWidth = 1.5 + pulse * 6
        ctx.strokeStyle = pulse > 0 ? `rgba(247,147,26,${0.35 + pulse * 0.65})` : 'rgba(200,210,230,0.35)'
        ctx.stroke()
      }

      st.ghosts = st.ghosts.filter((g) => ghostPosition(g, now).t < 1)
      for (const ghost of st.ghosts) {
        const pos = ghostPosition(ghost, now)
        drawNode(ctx, { colors: ghost.colors, known: false, addresses: [] }, pos.x, pos.y, ghost.r * (1 - pos.t * 0.6), pos.alpha)
      }

      for (const node of st.nodes) drawNode(ctx, node, node.x, node.y, node.r)

      ctx.font = `700 ${Math.max(13, 18 / cam.k)}px system-ui, sans-serif`
      ctx.textAlign = 'center'
      ctx.textBaseline = 'top'
      for (const node of st.nodes) {
        if (!node.label) continue
        ctx.lineWidth = 4 / cam.k
        ctx.strokeStyle = 'rgba(0,0,0,0.85)'
        ctx.fillStyle = '#fff'
        ctx.strokeText(node.label, node.x, node.y + node.r + 6)
        ctx.fillText(node.label, node.x, node.y + node.r + 6)
      }
    }
    frame = requestAnimationFrame(draw)

    return () => {
      cancelAnimationFrame(frame)
      window.removeEventListener('resize', resize)
      simulation.stop()
      st.simulation = null
    }
  }, [])

  useEffect(() => {
    const st = stateRef.current
    if (!data || !st.simulation) return
    const canvas = canvasRef.current
    const parent = canvas && canvas.parentElement
    const next = reconcile(
      { nodes: st.nodes, ghosts: st.ghosts },
      data,
      { width: (parent && parent.clientWidth) || 1280, height: (parent && parent.clientHeight) || 720, seenEdges: st.seenEdges },
    )
    st.nodes = next.nodes
    st.links = next.links
    st.ghosts = next.ghosts
    st.simulation.nodes(st.nodes)
    st.simulation.force('link').links(st.links)
    st.simulation.alpha(0.7).restart()
  }, [data])

  return (
    <div className={[styles.graphWrap, className].filter(Boolean).join(' ')}>
      <canvas ref={canvasRef} className={styles.canvas} data-testid="cluster-graph" />
    </div>
  )
}
