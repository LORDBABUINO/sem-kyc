import { expect, test } from 'vitest'
import { pickApp } from './routes.jsx'
import PhoneApp from './apps/phone/PhoneApp.jsx'
import ProjectorApp from './apps/projector/ProjectorApp.jsx'
import PresenterApp from './apps/presenter/PresenterApp.jsx'

test('routes by pathname', () => {
  expect(pickApp('/')).toBe(PhoneApp)
  expect(pickApp('')).toBe(PhoneApp)
  expect(pickApp('/qualquer-coisa')).toBe(PhoneApp)
  expect(pickApp('/projetor')).toBe(ProjectorApp)
  expect(pickApp('/projetor/')).toBe(ProjectorApp)
  expect(pickApp('/painel')).toBe(PresenterApp)
})
