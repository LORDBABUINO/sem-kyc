export const SESSION_KEY = 'dever.session'
export const ADMIN_KEY = 'dever.admin'
export const TAB_KEY = 'dever.tab'

export const storage = {
  get(key) {
    try {
      return window.localStorage.getItem(key)
    } catch {
      return null
    }
  },
  set(key, value) {
    try {
      window.localStorage.setItem(key, value)
    } catch {
      /* armazenamento indisponível: segue sem persistir */
    }
  },
  remove(key) {
    try {
      window.localStorage.removeItem(key)
    } catch {
      /* idem */
    }
  },
}
