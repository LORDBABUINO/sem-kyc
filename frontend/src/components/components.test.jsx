import { afterEach, describe, expect, test, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import QrCode from './QrCode.jsx'
import QrScanner from './QrScanner.jsx'
import Amount from './Amount.jsx'
import SectorBadge from './SectorBadge.jsx'
import VulnerabilityBadge from './VulnerabilityBadge.jsx'
import { PADARIA } from '../test/fixtures.js'

vi.mock('jsqr', () => ({ default: vi.fn(() => ({ data: 'ABC123' })) }))

afterEach(() => {
  vi.restoreAllMocks()
  delete navigator.mediaDevices
})

describe('QrCode', () => {
  test('renders an svg with modules for the value', () => {
    render(<QrCode value="bitcoin:bc1qabc?amount=0.0006" label="QR do pedido" />)
    const svg = screen.getByRole('img', { name: 'QR do pedido' })
    expect(svg.tagName.toLowerCase()).toBe('svg')
    expect(svg.getAttribute('data-value')).toBe('bitcoin:bc1qabc?amount=0.0006')
    expect(svg.querySelector('path').getAttribute('d')).toMatch(/^M\d+ \d+h1v1h-1z/)
  })

  test('renders nothing without a value', () => {
    const { container } = render(<QrCode value="" />)
    expect(container.innerHTML).toBe('')
  })
})

describe('QrScanner', () => {
  test('reports when the camera is unavailable', async () => {
    const onError = vi.fn()
    render(<QrScanner onScan={() => {}} onError={onError} />)
    await waitFor(() => expect(onError).toHaveBeenCalled())
    expect(screen.getByText(/Digite o código/)).toBeTruthy()
  })

  test('decodes a frame and stops the stream', async () => {
    const stop = vi.fn()
    navigator.mediaDevices = {
      getUserMedia: vi.fn(async () => ({ getTracks: () => [{ stop }] })),
    }
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({
      drawImage: () => {},
      getImageData: () => ({ data: new Uint8ClampedArray(4), width: 1, height: 1 }),
    })
    vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue()
    Object.defineProperty(HTMLMediaElement.prototype, 'readyState', { configurable: true, get: () => 4 })
    Object.defineProperty(HTMLVideoElement.prototype, 'videoWidth', { configurable: true, get: () => 320 })
    Object.defineProperty(HTMLVideoElement.prototype, 'videoHeight', { configurable: true, get: () => 240 })
    const onScan = vi.fn()
    const { unmount } = render(<QrScanner onScan={onScan} />)
    await waitFor(() => expect(onScan).toHaveBeenCalledWith('ABC123'))
    unmount()
    expect(stop).toHaveBeenCalled()
  })
})

describe('small components', () => {
  test('Amount formats sats', () => {
    render(<Amount value={49731} />)
    expect(screen.getByText('49.731 sats')).toBeTruthy()
  })

  test('SectorBadge shows the sector name and color', () => {
    render(<SectorBadge sector={PADARIA} />)
    const label = screen.getByText('Padaria')
    expect(label.parentElement.getAttribute('style')).toContain('#e0a030')
  })

  test('VulnerabilityBadge translates type and severity', () => {
    render(<VulnerabilityBadge type="ADDRESS_REUSE" severity="high" />)
    expect(screen.getByText('Reuso de endereço')).toBeTruthy()
    expect(screen.getByText('alta').getAttribute('data-severity')).toBe('high')
  })

  test('VulnerabilityBadge falls back for unknown types', () => {
    render(<VulnerabilityBadge type="SOMETHING_NEW" />)
    expect(screen.getByText('something new')).toBeTruthy()
  })
})
