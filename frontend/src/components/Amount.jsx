import { formatSats } from '../utils/format.js'

export default function Amount({ value, className }) {
  return <span className={className}>{formatSats(value)}</span>
}
