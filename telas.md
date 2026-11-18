# As três telas e o backend

Complemento de `roteiro.md`. Descreve o que cada tela mostra, o que o jogador pode
fazer em cada rodada, e o que o backend precisa guardar e emitir.

## 1. Celular do jogador

Web app, uma página, sem instalação. Abas na barra inferior, nesta ordem:
Missão, Carteira, Vender, Histórico, Crachá. A engrenagem de configurações fica no
canto da Carteira. Nenhuma tela do celular mostra finding, severidade ou aviso de
risco. A única pista de erro é o auto de infração chegando.

### Entrada

1. QR do projetor abre a URL. Pede um nome fantasia. O backend atribui o setor em
   rodízio e devolve cor, ícone e saldo inicial.
2. Tela de boas-vindas da cooperativa: "Você é a Padaria da Lu. Pague a taxa de
   cadastro da vitrine pra abrir as portas." Botão "pagar 10.000 sats". É a primeira
   transação do jogador e registra o endereço da vitrine.
3. Sessão gravada no navegador. Se a pessoa fechar a aba, volta no mesmo lugar.

### Missão

Mensagem da cooperativa da rodada atual (texto em `economia/missoes.md`), com os
dois itens a comprar, o setor fornecedor de cada um, a cor do setor, o preço
corrente e o bônus. Cada item tem um estado: a comprar, pago, expirado. Abaixo, o
timer da rodada. Quando os dois itens estão pagos, a linha "bônus da cooperativa:
chegou" aparece quando a transação do bônus entra.

Quando o palestrante avança a edição, esta aba recebe a mensagem nova e um cartão
"novidade na carteira" com uma frase sobre o que destravou e um botão que leva até
a configuração. Só uma frase. A aula é no projetor.

### Situação irregular (auto de infração)

Quando o jogador é autuado no reveal, o app inteiro ganha uma faixa vermelha no
topo: "Inscrição estadual suspensa. Auto de Infração DOE nº 12: ICMS devido 43.200
sats." Os botões de pagar e de vender ficam desabilitados com a legenda "regularize
pra operar". Pra pagar, o jogador escaneia o QR do guichê no projetor; a faixa também
traz o botão "pagar no guichê" como cópia, pra quem não alcança a tela. A tela de
pagamento do auto é a mesma de qualquer pagamento, com seleção de moeda: é uma
transação real pro endereço do DOE. Paga, a faixa some e os botões voltam.
Dívida que atravessa uma rodada aparece com a multa de mora somada.

### Carteira

Saldo total e a lista de UTXOs, cada um com valor, tipo de endereço (legacy,
segwit, Taproot), e, a partir da rodada 5, a origem (venda pra Fulano, bônus,
saque de exchange, cashback, troco). Ações por UTXO conforme destravado:

| Ação | Desde | O que faz |
|---|---|---|
| Congelar / descongelar | rodada 4 | some da seleção de pagamento |
| Sacar via Lightning | rodada 5 | só em UTXO de saque de exchange ainda não gasto; sai e volta como UTXO novo sem ligação |
| Entrar no CoinJoin | rodada 6 | escolhe o UTXO e entra na sala; fecha com 5 ou mais em 30 segundos, denominação 100.000, excedente volta como troco com aviso |
| Migrar pra Taproot | rodada 7 | gasta os UTXOs marcados pra endereços bc1p novos; aviso quando marca mais de um ("juntar liga") |

Configurações (engrenagem), com o padrão mudando quando a proteção destrava:

| Configuração | Opções | Padrão inicial | Destrava |
|---|---|---|---|
| Modo de recebimento | vitrine, endereço novo, Payjoin, Silent Payments | vitrine | novo na rodada 1, Payjoin e sp1 na rodada 8 |
| Estilo de preço | redondo, quebrado | redondo | rodada 3 |
| Seleção de moeda | automática (junta até cobrir), menor que cobre, manual | automática | rodada 2 |

O jogador pode voltar pra opção pior a qualquer momento. O jogo não impede.

### Vender

Catálogo do setor: os itens que o setor vende, com o preço corrente da rodada. O
vendedor toca no item que o comprador pediu e a tela vira o QR, grande, com o valor
e o item embaixo e um código de 6 caracteres pra digitação manual. O QR codifica
um URI de pagamento:

```
bitcoin:<endereço>?amount=<sats>&label=<item>&pj=<url do pedido>
```

O endereço vem do modo de recebimento. Em Silent Payments o endereço é o sp1. O
parâmetro `pj` só vai se o modo for Payjoin. O pedido é de uso único, expira no fim
da rodada, e a tela volta pro catálogo quando o pagamento entra, com um "recebido
de Fulano, 47.300 sats" por dois segundos.

Se o modo for Payjoin, quando o comprador propõe, a tela do vendedor mostra
"Padaria da Lu quer pagar junto. Contribuir uma moeda?" com a lista de UTXOs pra
escolher e um botão de aceitar. Se recusar, a transação sai normal.

### Pagar

Entra pelo botão "escanear" da aba Missão (ou digitando o código). Mostra item,
vendedor, valor. Abaixo, a seleção de moeda conforme a configuração: automática
não mostra nada; manual mostra a lista de UTXOs com checkbox, o total selecionado
e o troco resultante. Se existe UTXO do valor exato, uma linha "pague sem troco com
esta" aparece a partir da rodada 3. Botão "pagar". Se o QR tem `pj` e o jogador
está na rodada 8, aparece "pagar junto (Payjoin)" como botão principal.

### Histórico

Lista de transações em ordem inversa, no formato de carteira comum: seta de
entrada ou saída, valor, contraparte quando conhecida pelo próprio jogador
("Transportadora do Zé", "cooperativa", "DOE"), item ou motivo curto, bloco.
O pagamento de um auto aparece como saída pro DOE com o texto "ICMS, Auto de
Infração nº N", sem o motivo. O cashback
aparece como entrada de 546 sats de "desconhecido". Toque em uma transação abre
inputs e outputs crus, com endereços, pra quem quiser olhar com os próprios olhos.

### Crachá

Tela cheia com a cor do setor, o ícone e o nome fantasia em fonte grande, pra
mostrar pra quem está procurando. Brilho máximo. É também a tela que se mostra pro
vendedor antes de pedir o produto.

### Relatório (só na edição final)

Botão novo na aba Missão. Abre o relatório do Stealth do jogador: findings e
warnings de todas as rodadas, agrupados por tipo, com severidade, evidência (txid,
endereços, valores) e a correção. Mesmo formato do frontend do Stealth, pra
reaproveitar componente.

## 2. Projetor (a visão do DOE)

Tela cheia, sem interação. Três estados, trocados pelo painel do apresentador.

### Jornal

Primeira página de O Rastro. Cabeçalho com nome, a linha "Órgão oficial do
Departamento de Ordem Econômica da República Democrática de Nova Alvorada", data fictícia ("Rodada 3"), bloco
atual. Manchete e linha fina em corpo grande. Portaria à direita, em caixa. Rodapé
com a apuração da rodada anterior: "17 estabelecimentos autuados pelo Portaria DOE nº 2,
ICMS lançado: 410.000 sats, quitado: 260.000", e uma coluna estreita com os nomes,
os valores e a situação (quitado ou em aberto) em corpo pequeno. Na edição 1 o rodapé diz "Cadastro concluído: N
estabelecimentos".

### Grafo ao vivo

Grafo de força, atualizado a cada transação pelo WebSocket, com:

- Nó = cluster, tamanho proporcional ao saldo que o DOE estima, cor da
  identidade quando rotulado, cinza quando não, listrado com as duas cores quando
  em conflito. Dentro do nó, pontos pequenos são os endereços.
- Aresta = transação entre clusters. Pulsa quando entra.
- Fusão animada: o nó cinza desliza e é absorvido pelo rotulado em meio segundo.
- Rótulo = nome fantasia, só em clusters rotulados sem conflito. Em conflito,
  "Padaria da Lu ou Fazenda do Caio?".
- Faixa superior: timer da negociação, pagamentos na rodada, bloco atual.
- Faixa inferior "Dívida Ativa": nomes dos autuados que ainda não pagaram e o
  valor, rolando. Presente em todas as cenas a partir do primeiro auto, e vazia
  quando todo mundo quitou. No canto, o QR do guichê do ICMS, fixo.
- Zoom automático: clusters rotulados ao centro, cinzas na periferia. Até uns
  1.500 endereços no fim do jogo; por isso o nó é o cluster e não o endereço.

Regras de fusão (union-find global, incremental, no backend):

1. Inputs da mesma transação pertencem ao mesmo cluster (CIOH), exceto se a
   transação tem a forma de CoinJoin (3 ou mais outputs do mesmo valor) ou foi
   marcada como Payjoin pelo próprio jogo. Em Payjoin a fusão acontece mesmo assim,
   porque o DOE não sabe que é Payjoin: é isso que cria o conflito.
2. Output identificado como troco entra no cluster dos inputs. Troco é: o output
   de valor não redondo quando o outro é redondo; o output do mesmo tipo de script
   dos inputs quando o outro difere; o único output quando há um só e ele não é o
   valor pedido em nenhum pedido conhecido do DOE (o DOE não conhece
   pedidos; só a primeira e a segunda regra valem).
3. Endereço reutilizado: todas as transações pro mesmo endereço entram no mesmo
   cluster.
4. Dust do DOE gasto junto com outros inputs: já coberto pela regra 1, mas o
   evento é marcado pra animação e pro rodapé.
5. Rótulos: endereço da vitrine cadastrado na rodada 0; a partir da rodada 5, o
   UTXO inicial da Loja de eletrônicos. Um cluster com dois rótulos diferentes é
   conflito.

### Reveal

Painel que entra sobre o grafo quando a negociação fecha:

- Zoom no cluster do caso da manchete seguinte (o finding mais grave da rodada,
  desempate aleatório; o backend escolhe entre os jogadores que têm o detector da
  rodada).
- Cartão "o que o DOE sabe": nome, operações identificadas na rodada contra as
  reais, número de endereços ligados, contrapartes identificadas (nomes), e o ICMS
  lançado.
- Lista de autuados da rodada: nome, valor devido, portaria que fundamenta. Ao lado,
  o QR do guichê do ICMS em tamanho grande. Total lançado na rodada.
- Se houve CoinJoin ou Payjoin na rodada, segunda cena mostrando a transação e o
  painel vazio ou em conflito.

## 3. Painel do apresentador

Página separada, protegida por senha, no celular ou no notebook. Botões grandes:

| Botão | Efeito |
|---|---|
| Abrir cadastro | projetor mostra o QR e o contador de entrada |
| Avançar edição | jornal da edição N no projetor, mensagem da cooperativa e destravas nos celulares |
| Abrir negociação | grafo ao vivo com timer de 3 minutos (editável) |
| Fechar e apurar | expira pedidos, lança o ICMS (gera os autos), paga os bônus, escolhe a manchete, mostra o reveal |
| Perdoar auto | cancela um auto específico (pra quando um bug autua errado ao vivo) |
| Cashback Cidadão | envia 546 sats pra todos os endereços conhecidos (edição 4) |
| Receber cadastro das exchanges | aplica os rótulos de KYC (edição 5) |
| Liberar relatório | botão Relatório nos celulares e placar no projetor (final) |
| Voltar pro grafo / pro jornal | troca a cena do projetor sem mudar de rodada |

Abaixo dos botões, o estado da sala: jogadores por setor, pagamentos na rodada,
quem ainda não completou a missão, autos em aberto (quem, quanto, há quantas
rodadas), sala de CoinJoin (quantos dentro, segundos restantes), e o log das últimas
transações com nomes. É o que você olha pra narrar
durante a negociação sem depender do grafo.

Opções: timer da rodada, percentual do ICMS (18%), multa de ofício (100%), multa de
mora (20% por rodada), alvo de receita por vendedor, modo bots (preenche
setores vazios e joga sozinho pra ensaio), reiniciar jogo.

## 4. Backend

Rust, `axum`, igual à API do Stealth, com `stealth-engine` e `stealth-model` como
dependências de workspace. Pode viver como crate do repositório do Stealth
(`stealth-game`) ou aqui, apontando pro repositório por path. Front em React com
Vite, sem TypeScript e sem Tailwind, pra reaproveitar componentes do frontend do
Stealth no relatório final. Grafo com d3-force.

### DOE

- `players`: id, nome, setor, cor, sessão, endereço da vitrine, configurações,
  missões por rodada com estado de cada item.
- `ledger`: transações em memória no formato que `TxGraph::from_wallet_history`
  consome (`DecodedTransaction`, `WalletHistory` por jogador). Carteiras internas:
  DOE, cooperativa, exchange (origem do saldo da Loja), minerador fictício
  (origem dos saldos iniciais).
- `blocks`: contador, fecha a cada 30 segundos, carimba as transações pendentes.
- `clusters`: union-find global com as regras da seção 2, incremental.
- `rounds`: edição atual, estado (jornal, negociação, apurado), preços da rodada,
  bônus pagos, manchete escolhida.
- `autos`: autos de infração: jogador, rodada, operações que fundamentam, portaria,
  valor (imposto mais multa de ofício, mais mora acumulada), estado (aberto, pago),
  txid do pagamento. Jogador com auto aberto não gera pedido nem paga pedido.
- `invoices`: pedidos abertos, código de 6 caracteres, vendedor, item, valor,
  endereço, modo, expiração.
- `coinjoin_room`: participantes, UTXOs, prazo.

### Eventos pelo WebSocket

Celular: `mission`, `unlock`, `invoice_paid`, `payjoin_request`, `tx`, `auto`
(novo auto ou quitação), `bonus`, `report_available`. Projetor: `scene`, `tx` (com a lista de fusões que
provocou), `cluster_label`, `timer`, `reveal`. Painel: tudo, mais `room_state`.

### Stealth

A cada transação, em segundo plano, o backend reconstrói o `WalletHistory` de cada
jogador afetado e roda `TxGraph::from_wallet_history` seguido de `detect_all` com
os thresholds padrão, passando os txids da exchange como `known_exchange_txids` a
partir da rodada 5. Os findings ficam guardados por rodada. São eles que:

- escolhem a manchete da edição seguinte (maior severidade entre os jogadores que
  acionaram o detector da rodada; desempate aleatório);
- compõem o relatório final de cada jogador;
- alimentam o cartão do reveal.

O lançamento do ICMS não usa os findings: pra cada pagamento da rodada, olha o
cluster do pagador e o cluster do output recebido no union-find. Ponta em cluster
rotulado sem conflito deve 18% do valor mais 100% de multa de ofício. Os autos de
um jogador na rodada são somados em um auto só. Assim o grafo e a cobrança nunca
discordem. O pagamento do auto é uma transação comum do jogador pro endereço do
guichê (fixo, reutilizado de propósito: o DOE não se esconde), e entra no
union-find como qualquer outra: as moedas usadas ganham o rótulo do pagador.

A taxa de rede do Portaria DOE nº 6 é narrativa: o ledger do jogo não cobra fee, pra
um pagamento no valor exato continuar sem troco.

### Preços

Antes de abrir cada negociação: pra cada setor vendedor, `compradores` = número de
jogadores cujas missões da rodada incluem um item daquele setor; `vendedores` =
jogadores do setor. Preço unitário = alvo × vendedores / compradores, arredondado
pra um valor quebrado (termina em dígitos aleatórios) se o estilo do vendedor for
quebrado, ou pro milhar mais próximo se for redondo. Com `compradores` zero, o
preço não importa e o setor fica sem venda na rodada.

### Modo bots

Um bot por setor vazio, mais os que o apresentador pedir. Bots compram as missões
com erro proposital configurável por rodada (reusam endereço até a rodada 1,
juntam moedas até a 2, gastam o dust na 4, e assim por diante) pra que o ensaio
dispare cada detector no slide certo. Bots nunca vendem pra humanos (humanos
precisam de humanos); vendem só pra bots.

### Hospedagem

Um binário, estático embutido. Nuvem com domínio e certificado é o caminho
principal, porque a câmera do navegador exige HTTPS. Plano B: o mesmo binário no
notebook servindo pelo hotspot, com certificado autoassinado instalado antes, ou
com a digitação do código de 6 caracteres como substituto do QR. O QR da abertura
é gerado na hora com a URL que estiver funcionando.
