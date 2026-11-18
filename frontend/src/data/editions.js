// Texto fixo de cada edição de O Fiscal, copiado de roteiro.md.
export const EDITIONS = {
  1: {
    headline: 'Receita publica o saldo de todas as lojas da cidade',
    subheadline:
      'Com o cadastro da vitrine, Estado afirma conhecer o patrimônio de cada comerciante em tempo real.',
    edital:
      'Toda operação comercial sem nota fiscal identificada pela Secretaria da Fazenda será autuada: ICMS de 18% sobre o valor da operação, mais multa de ofício de 100% do imposto. A inscrição estadual do autuado fica suspensa até a quitação.',
  },
  2: {
    headline: 'Endereços novos eram da mesma dona, diz cruzamento de dados',
    subheadline:
      "Técnica de análise liga endereços 'anônimos' a comerciantes cadastrados a partir de um único pagamento.",
    edital:
      'O Estado passa a aplicar a Presunção de Propriedade Comum: moedas gastas juntas pertencem ao mesmo contribuinte.',
  },
  3: {
    headline: 'Estado mapeia o troco e descobre fornecedores',
    subheadline:
      "Receita identifica 'a moeda que volta' e remonta a cadeia de pagamentos de cada estabelecimento.",
    edital:
      'Valores de referência: o comércio deve praticar preços redondos para facilitar a fiscalização.',
  },
  4: {
    headline: 'Programa Cashback Cidadão beneficia todas as lojas',
    subheadline:
      "Cada estabelecimento recebe 546 sats de incentivo. 'Um gesto de boa vontade', diz o Estado.",
    edital:
      'Os 546 sats do Cashback Cidadão serão creditados em todos os endereços conhecidos de cada contribuinte.',
  },
  5: {
    headline: 'Exchanges passam a entregar o cadastro ao Estado',
    subheadline:
      'Acordo de cooperação revela o nome por trás de todo saque. Loja de eletrônicos é o setor mais afetado.',
    edital: 'Toda moeda originada em exchange passa a carregar o nome do titular da conta.',
  },
  6: {
    headline: 'Consolidação revela patrimônio de grande produtor',
    subheadline: 'Alta nas taxas leva comerciantes a juntar moedas. Estado agradece.',
    edital: 'Em razão do congestionamento, a taxa de rede sobe para 2.000 sats por input.',
  },
  7: {
    headline: 'Estado cataloga carteiras pela assinatura',
    subheadline:
      "Tipo de endereço, ordem dos outputs e horário formam a 'digital' de cada estabelecimento.",
    edital:
      'A Secretaria da Fazenda passa a considerar o tipo de endereço e o padrão de gasto como indício de titularidade para fins de lançamento do ICMS.',
  },
  8: {
    headline: 'Recadastramento da vitrine: endereço fixo volta a ser obrigatório',
    subheadline:
      "Estado exige que todo comerciante publique um endereço único de recebimento. 'Fim da bagunça dos endereços descartáveis', comemora Receita.",
    edital:
      "Todo estabelecimento deve cadastrar um endereço fixo de recebimento até o fim da rodada. Fica também proibida a 'presunção falha': transações com inputs de dois titulares serão investigadas.",
  },
}

export const FINAL_EDITION = {
  headline: 'Estado liberta os arquivos',
  subheadline:
    'Por decisão judicial, cada contribuinte recebe o dossiê que o Estado mantinha sobre ele.',
  takeaways: [
    'Endereço novo sempre.',
    'Uma moeda por pagamento.',
    'Não gaste o que não pediu.',
    'Não misture origens.',
    'Juntar só em CoinJoin.',
    'Publique sp1.',
  ],
}

export function editionFor(round) {
  return EDITIONS[round] || null
}
