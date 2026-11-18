import React from 'react'
import { createRoot } from 'react-dom/client'
import './styles/global.css'
import { pickApp } from './routes.jsx'

const App = pickApp(window.location.pathname)
createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
