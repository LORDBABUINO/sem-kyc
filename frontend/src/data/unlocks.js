// Uma frase por destrava, encurtada dos parágrafos "Cooperativa:" do roteiro.md.
// target: onde o botão do cartão leva ("settings", "wallet" ou "report").
export const UNLOCKS = {
  fresh_address: {
    sentence: 'Seu QR agora muda a cada venda: não receba mais no endereço da vitrine.',
    target: 'settings',
    setting: 'receive_mode',
    button: 'ver modo de recebimento',
  },
  coin_control: {
    sentence: 'Agora você escolhe com qual moeda paga: uma moeda por pagamento, e no guichê use a que o DOE já conhece.',
    target: 'settings',
    setting: 'coin_selection',
    button: 'ver seleção de moeda',
  },
  odd_price: {
    sentence: 'Seu preço agora sai quebrado, e na hora de pagar prefira uma moeda do valor exato.',
    target: 'settings',
    setting: 'price_style',
    button: 'ver estilo de preço',
  },
  freeze: {
    sentence: 'Chegou um presente do DOE na sua carteira: não gaste, congele e deixe lá pra sempre.',
    target: 'wallet',
    button: 'abrir carteira',
  },
  lightning: {
    sentence: 'Moeda de saque de exchange tem seu nome desde que nasceu: passe pela Lightning antes de gastar.',
    target: 'wallet',
    button: 'abrir carteira',
  },
  coin_join: {
    sentence: 'Não junte moedas sozinho: vá até a mesa do CoinJoin, com 5 pessoas a mistura fecha.',
    target: 'wallet',
    button: 'abrir carteira',
  },
  taproot: {
    sentence: 'Migre pra Taproot uma moeda de cada vez, nunca todas juntas, e só receba em bc1p.',
    target: 'wallet',
    button: 'abrir carteira',
  },
  payjoin: {
    sentence: 'Nas compras desta rodada, peça pra pagar junto: o Payjoin confunde a presunção do DOE.',
    target: 'settings',
    setting: 'receive_mode',
    button: 'ver modo de recebimento',
  },
  silent_payments: {
    sentence: 'Dê ao DOE um sp1: é fixo, pode imprimir, e cada pagamento cai num lugar diferente.',
    target: 'settings',
    setting: 'receive_mode',
    button: 'ver modo de recebimento',
  },
  report: {
    sentence: 'Seu relatório está pronto: é o dossiê que o DOE mantinha sobre você.',
    target: 'report',
    button: 'abrir relatório',
  },
}

export function unlockInfo(unlock) {
  return UNLOCKS[unlock] || null
}
