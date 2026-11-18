import PhoneApp from './apps/phone/PhoneApp.jsx'
import ProjectorApp from './apps/projector/ProjectorApp.jsx'
import PresenterApp from './apps/presenter/PresenterApp.jsx'

export function pickApp(pathname) {
  const path = String(pathname || '/').replace(/\/+$/, '') || '/'
  if (path === '/projetor') return ProjectorApp
  if (path === '/painel') return PresenterApp
  return PhoneApp
}
