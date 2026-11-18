export const VULNERABILITY_LABELS = {
  ADDRESS_REUSE: 'Reuso de endereço',
  CIOH: 'Inputs em comum',
  DUST: 'Dust recebido',
  DUST_SPENDING: 'Dust gasto',
  CHANGE_DETECTION: 'Troco identificado',
  CONSOLIDATION: 'Consolidação',
  SCRIPT_TYPE_MIXING: 'Mistura de tipos de script',
  CLUSTER_MERGE: 'Fusão de clusters',
  UTXO_AGE_SPREAD: 'Idades de UTXO misturadas',
  EXCHANGE_ORIGIN: 'Origem em exchange',
  TAINTED_UTXO_MERGE: 'Mistura com moeda marcada',
  BEHAVIORAL_FINGERPRINT: 'Assinatura comportamental',
  DORMANT_UTXOS: 'UTXOs dormentes',
  DIRECT_TAINT: 'Contaminação direta',
  DUST_ATTACK: 'Ataque de dust',
  PEEL_CHAIN: 'Peel chain',
  DETERMINISTIC_LINK: 'Ligação determinística',
  UNNECESSARY_INPUT: 'Input desnecessário',
  TOXIC_CHANGE: 'Troco tóxico',
}

export const SEVERITY_LABELS = {
  critical: 'crítica',
  high: 'alta',
  medium: 'média',
  low: 'baixa',
  info: 'informativa',
}

export function vulnerabilityLabel(type) {
  if (!type) return ''
  return VULNERABILITY_LABELS[type] || String(type).toLowerCase().replace(/_/g, ' ')
}

export function severityLabel(severity) {
  const key = String(severity || '').toLowerCase()
  return SEVERITY_LABELS[key] || key
}
