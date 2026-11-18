// A mission or invoice item: an object from the API, a plain string in old fixtures.
export function itemName(item) {
  return typeof item === 'string' ? item : (item && item.name) || ''
}
