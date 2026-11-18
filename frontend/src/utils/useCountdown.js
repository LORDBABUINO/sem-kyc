import { useEffect, useState } from 'react'
import { secondsLeft } from './format.js'

export function useCountdown(deadline) {
  const [left, setLeft] = useState(() => secondsLeft(deadline))
  useEffect(() => {
    setLeft(secondsLeft(deadline))
    if (deadline == null) return undefined
    const timer = setInterval(() => setLeft(secondsLeft(deadline)), 500)
    return () => clearInterval(timer)
  }, [deadline])
  return left
}
