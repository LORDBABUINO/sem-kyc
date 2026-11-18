# O Rastro: roteiro da palestra-jogo

Palestra interativa de 90 minutos sobre o funcionamento da rede Bitcoin e privacidade
nas transações. A plateia entra por QR code, cada pessoa vira um empreendedor de um
setor e compra insumos dos outros pagando em bitcoin. O palestrante é o DOE: lê
a chain, publica um jornal a cada rodada, lança ICMS sobre quem ele consegue identificar e
explica, do lado de quem vigia, o que não fazer.

Arquivos relacionados: `telas.md` (as três telas e o backend), `economia/trilhas.json`
(o que cada setor compra em cada rodada), `economia/missoes.md` (as 64 mensagens).

## 1. Premissas do jogo

**Personas.** Oito setores, distribuídos em rodízio na ordem de entrada:
Padaria, Fazenda, Energia solar, Transportadora, Loja de eletrônicos, Construtora,
Consultoria, Gráfica. Cada setor tem uma cor, um ícone e um adesivo entregue na
entrada. O jogador escolhe só o nome fantasia ("Padaria da Lu").

**Saldo inicial por setor.** Mesmo valor total, formatos diferentes, porque o
formato já carrega a vulnerabilidade de partida.

| Setor | Saldo inicial | Forma | Vulnerabilidade embutida |
|---|---|---|---|
| Padaria | 2.000.000 sats | 20 UTXOs de ~100.000 | tentação de consolidar |
| Fazenda | 2.000.000 | 2 UTXOs de 1.000.000, um legacy (1...) e um segwit (bc1q) | troco grande, mistura de script |
| Energia solar | 2.000.000 | 4 UTXOs de 500.000 | neutro |
| Transportadora | 2.000.000 | 8 UTXOs de 250.000 | neutro |
| Loja de eletrônicos | 2.000.000 | 1 UTXO de 2.000.000 vindo de saque de exchange | KYC (revelado na rodada 5) |
| Construtora | 2.000.000 | 3 UTXOs de ~666.000 | troco grande |
| Consultoria | 2.000.000 | 1 UTXO de 2.000.000 | troco grande, peel chain |
| Gráfica | 2.000.000 | 16 UTXOs de 125.000, dois deles legacy | consolidar, mistura de script |

**Economia.** Cada jogador compra 2 insumos por rodada, sempre de setores
específicos (`economia/trilhas.json`). o DOE alvo por vendedor igual em toda
rodada: o preço unitário é o alvo dividido pelo número de setores compradores daquele
vendedor na rodada, corrigido pelo número real de jogadores em cada setor. Alvo
sugerido: 120.000 sats por vendedor por rodada, o que dá preços entre 40.000 e
120.000. Quem completa as duas compras dentro do timer recebe da cooperativa um
bônus de 25% do que gastou, em uma transação da carteira da cooperativa.

**Vender é presencial.** O comprador anda pela sala, acha o fornecedor pelo adesivo,
pede o produto. O vendedor toca no produto, o celular mostra o QR com o pedido de
pagamento. O comprador escaneia, escolhe os UTXOs se já puder, confirma. O pedido é
de uso único e expira no fim da rodada. Não há estoque: qualquer vendedor de um
setor vende qualquer item do catálogo do setor pelo mesmo preço.

**Blocos.** O backend fecha um bloco a cada 30 segundos. O item é entregue no
broadcast (o DOE também olha o mempool). O contador de blocos fica no projetor
o tempo todo, pra conversa de abertura ter onde apontar.

**O que o DOE sabe.** Só três coisas: a chain inteira, o endereço da vitrine que
cada jogador cadastrou na rodada 0, e, a partir da rodada 5, o cadastro da
exchange. Tudo o mais ele deduz com as heurísticas (o grafo de clusters, ver
`telas.md`).

**ICMS.** O imposto do jogo é o ICMS, o imposto estadual sobre circulação de
mercadorias e serviços. Toda venda do jogo é uma operação sem nota, e o DOE
lança o imposto sobre as operações que consegue identificar. No reveal de cada
rodada, pra cada pagamento da rodada: se o cluster de quem pagou está rotulado com
uma identidade só, quem pagou deve 18% do valor; se o output de quem recebeu está em
cluster rotulado, quem recebeu deve 18%. Soma-se a multa de ofício de 100% do
imposto por operação sem nota, como fazem as fazendas estaduais, então cada ponta
identificada deve 36% do valor da operação. Cluster sem rótulo não gera lançamento.
Cluster com dois rótulos (conflito, efeito do Payjoin) não gera lançamento. Os dois
percentuais são parâmetros do painel.

**O auto de infração não é automático.** No reveal o DOE expõe os autuados no
projetor: nome fantasia, valor devido e o portaria que fundamenta. Ao lado fica o QR do
guichê do ICMS. O autuado escaneia o QR do projetor (ou toca na cópia que aparece no
celular) e paga o auto com uma transação real pro endereço do DOE. Enquanto não
paga, a inscrição estadual fica suspensa: o app não deixa comprar nem vender. O nome
fica rolando na faixa "Dívida Ativa" do projetor até o pagamento entrar. Dívida que
atravessa uma rodada ganha multa de mora de 20% por rodada, e no placar final o
saldo é descontado da dívida em aberto, pra ninguém ganhar parando de jogar. No
histórico o pagamento aparece como saída pro DOE com o texto "ICMS, Auto de
Infração nº N", sem o motivo.

**Pagar o auto também é uma transação.** O DOE sabe quem está pagando, então toda
moeda usada no pagamento ganha o nome do pagador. Quem paga com uma moeda limpa
entrega a moeda. A dica, que a cooperativa só dá na rodada 2, é pagar o auto com as
moedas que já estavam expostas. Não há lacre nem expulsão.

**Placar.** Saldo final em sats. Um número só.

**Painel do Stealth.** Nunca no celular durante o jogo. Na edição final o botão
"Relatório" aparece em todos os celulares com os findings de todas as rodadas e a
correção de cada um.

## 2. Linha do tempo

| Minuto | Bloco |
|---|---|
| 0 a 15 | Abertura: como a rede funciona, enquanto a plateia entra e faz o cadastro |
| 15 a 71 | 8 rodadas de 7 minutos: 3 de jornal e explicação, 3 de negociação, 1 de reveal (os autuados pagam o auto durante o jornal seguinte) |
| 71 a 86 | Edição final: relatório liberado, placar, o que levar pra casa |
| 86 a 90 | Perguntas e folga |

Ritmo de cada rodada:

1. Você avança o slide. O projetor mostra a primeira página de O Rastro. Ao mesmo
   tempo chega no celular a mensagem da cooperativa com a missão e o botão novo.
2. Três minutos: você fala como DOE. Manchete, portaria e o mecanismo por trás.
3. Você toca "abrir negociação". O projetor troca pro grafo de clusters ao vivo com
   o timer de 3 minutos e o contador de pagamentos.
4. Timer zera. Pedidos abertos expiram. O backend lança o ICMS e paga os bônus,
   escolhe a manchete da próxima edição. O projetor dá zoom no cluster do caso mais
   grave, mostra o painel "o que o DOE sabe", a lista de autuados e o QR do
   guichê. Um minuto. Os autuados escaneiam e pagam durante o jornal da edição
   seguinte, antes da próxima negociação abrir.
5. Você avança pra próxima edição.

## 3. Abertura (15 minutos)

Projetor: QR gigante, contador de blocos, contador de pessoas cadastradas.

Ao entrar, o jogador dá um nome fantasia, recebe o setor e o saldo inicial, e o
app pede a primeira transação: pagar 10.000 sats de "taxa de cadastro da vitrine"
pro endereço do DOE. Ao pagar, o endereço de recebimento dele fica registrado
como endereço da vitrine. Essa é a transação que faz todo mundo nascer rotulado.

Fala do DOE, em ordem, com o grafo vazio ganhando os primeiros nós conforme as
pessoas cadastram:

1. **Rede.** Milhares de computadores guardando a mesma lista de transações. Não
   tem servidor, não tem dono, e, importante pra hoje, não tem segredo: toda
   transação é pública pra sempre. Apontar pro contador de blocos: a cada 30
   segundos aqui (10 minutos na rede real) um bloco fecha e vira história.
2. **Endereço.** Um endereço é um cofre com janela de vidro: qualquer um vê quanto
   tem e quando entrou e saiu. O que ninguém vê é de quem é. Toda a privacidade do
   Bitcoin mora nessa frase. O trabalho do DOE é ligar o endereço ao nome.
3. **UTXO.** Bitcoin não tem saldo, tem moedas de valores arbitrários. O saldo que
   o app mostra é a soma das moedas que você consegue gastar. Mostrar a carteira
   de um jogador no projetor (sem nome): "esta Fazenda tem 2 moedas de 1.000.000".
4. **Transação.** Gastar é derreter moedas (inputs) e fundir moedas novas
   (outputs). Se a moeda é maior que o pagamento, sobra troco, que volta pra você
   em um output. Toda transação tem essa cara: inputs de um lado, outputs do outro.
5. **O que o DOE vê.** Apontar pro grafo: cada nó é um endereço, cada linha é
   uma transação. Hoje o DOE sabe o nome de um endereço de cada um de vocês, o da
   vitrine, porque vocês acabaram de cadastrar. O jogo é descobrir o resto.
6. **Regras em uma frase cada.** Duas compras por rodada, presencial, QR do
   vendedor. Bônus de quem completa. ICMS sobre toda operação que o DOE
   consegue ligar ao seu nome, e quem é autuado só volta a operar depois de pagar
   no guichê. Ganha quem termina com mais sats. Nada no celular diz se você errou.

## 4. As edições

Cada edição tem: manchete e linha fina, portaria, a fala do DOE em 3 minutos
(mecanismo e o que não fazer), o que destrava no celular, a mensagem da cooperativa
(a parte da proteção; a parte da missão está em `economia/missoes.md`), o reveal e
as ações do apresentador.

### Edição 1: "DOE publica o saldo de todas as lojas da cidade"

Linha fina: "Com o cadastro da vitrine, DOE afirma conhecer o patrimônio de cada
comerciante em tempo real."

Portaria DOE nº 1: "Toda operação comercial sem nota fiscal identificada pela
DOE será autuada: ICMS de 18% sobre o valor da operação, mais
multa de ofício de 100% do imposto. A inscrição estadual do autuado fica suspensa
até a quitação."

Fala do DOE:
- O endereço da vitrine é público e fixo. Cada venda que cai nele soma na janela
  de vidro. O DOE nem precisa de heurística: é ler o saldo.
- Pior: quem paga a vitrine também fica ligado a ela. O fornecedor da Padaria está
  a uma seta de distância no grafo.
- O que não fazer: receber duas vezes no mesmo endereço. Um endereço reutilizado é
  um extrato bancário publicado no muro.
- Pra plateia: "na vida real isso é o QR impresso no balcão, a doação fixa no
  rodapé do site, o endereço colado na bio".
- Avisar que a partir de agora a carteira gera endereço novo a cada venda, e que o
  DOE "lamenta a medida".
- Explicar o auto de infração antes de abrir a negociação: quem for autuado vê o
  nome no projetor e só volta a operar depois de escanear o guichê e pagar.

Destrava: modo de recebimento "endereço novo por venda" (vira padrão ao destravar;
quem quiser pode voltar pra vitrine nas configurações, e o jogo não impede).

Cooperativa: "A partir de hoje seu QR muda a cada venda. O endereço da vitrine
continua cadastrado, então não receba mais nele. Nada liga um endereço novo a você,
a menos que você ligue."

Reveal: zoom no cluster do jogador que mais recebeu na vitrine nesta rodada.
Painel: operações identificadas na rodada, ICMS lançado, fornecedores identificados.
Primeira vez que a lista de autuados e o QR do guichê aparecem: deixar a plateia
entender que o jogo trava pra quem não paga.
Detector: ADDRESS_REUSE.

Apresentador: botão "abrir negociação", botão "fechar e apurar".

### Edição 2: "Endereços novos eram da mesma dona, diz cruzamento de dados"

Linha fina: "Técnica de análise liga endereços 'anônimos' a comerciantes
cadastrados a partir de um único pagamento."

Portaria DOE nº 2: "O DOE passa a aplicar a Presunção de Propriedade Comum: moedas
gastas juntas pertencem ao mesmo contribuinte."

Fala do DOE:
- Uma transação só pode ser assinada por quem tem a chave de cada input. Se uma
  transação gasta três moedas, as três são da mesma pessoa (CIOH).
- Então basta uma moeda conhecida na transação pra todas as outras ganharem nome.
  O endereço novo da rodada passada virou endereço da Padaria no instante em que
  foi gasto junto com o da vitrine.
- Mostrar no grafo ao vivo uma fusão da rodada anterior, se houve.
- O que não fazer: deixar a carteira escolher sozinha. Carteira comum junta moedas
  pra fechar o valor e junta sua história com elas. Pagar 60.000 com uma moeda de
  100.000 liga menos coisa do que com duas de 30.000 e uma de 10.000.
- Ressalva honesta do DOE: "a presunção falha às vezes, e o DOE não se
  importa".

Destrava: coin control na tela de pagamento (lista de UTXOs com valor e origem, o
jogador marca quais entram; padrão passa a ser "a menor moeda que cobre").

Cooperativa: "Agora você escolhe com qual moeda paga. Regra de bolso: uma moeda por
pagamento, e nunca uma moeda que já tem seu nome junto com uma que não tem. Isso
vale também pro guichê do ICMS: o DOE sabe quem está pagando o auto, então pague
com a moeda que ele já conhece, nunca com uma limpa."

Reveal: zoom na maior fusão da rodada (mais endereços engolidos de uma vez).
Detectores: CIOH, UNNECESSARY_INPUT.

### Edição 3: "DOE mapeia o troco e descobre fornecedores"

Linha fina: "DOE identifica 'a moeda que volta' e remonta a cadeia de
pagamentos de cada estabelecimento."

Portaria DOE nº 3: "Valores de referência: o comércio deve praticar preços redondos para
facilitar a fiscalização." (É sátira, e a plateia pode seguir se quiser.)

Fala do DOE:
- Quase toda transação tem dois outputs: o pagamento e o troco. O troco volta pra
  quem pagou. Se o DOE descobre qual output é o troco, continua seguindo você.
- Como descobre: valor redondo é pagamento (gente escolhe 50.000), valor quebrado é
  troco (ninguém escolhe 23.417). Output do mesmo tipo de endereço do input é
  troco. Output que depois é gasto junto com outras moedas suas é troco.
- Seguir troco de troco de troco é o peel chain: uma moeda grande sendo descascada
  em pagamentos, e o DOE anotando cada casca.
- O que não fazer: pagar valor redondo com moeda grande. E, pra quem vende, cobrar
  valor redondo, porque isso entrega o troco do seu cliente.
- Olhar pra Consultoria e Fazenda: "quem começou com uma moeda de 2.000.000 está
  descascando ela desde a rodada 1".

Destrava: no vendedor, "estilo de preço" (redondo ou quebrado; quebrado vira
padrão). No comprador, aviso na tela de pagamento quando a moeda escolhida é
exatamente o valor (sem troco) e sugestão de moeda sem troco quando existe.

Cooperativa: "Seu preço agora sai quebrado, e isso protege quem compra de você. Na
hora de pagar, se tiver uma moeda do valor exato, use ela: sem troco, sem rastro."

Reveal: zoom no peel chain mais longo da rodada, animado casca por casca.
Detectores: CHANGE_DETECTION, PEEL_CHAIN.

### Edição 4: "Programa Cashback Cidadão beneficia todas as lojas"

Linha fina: "Cada estabelecimento recebe 546 sats de incentivo. 'Um gesto de boa
vontade', diz o DOE."

Portaria DOE nº 4: "Os 546 sats do Cashback Cidadão serão creditados em todos os
endereços conhecidos de cada contribuinte."

Fala do DOE:
- Hoje o DOE dá dinheiro. Apertar o botão: 546 sats caem em todos os endereços
  que o DOE já ligou a alguém e também nos cinzas que ele quer ligar.
- Por que: a moeda é tão pequena que a carteira comum gasta ela junto com outras
  sem ninguém perceber. No instante em que gasta, a presunção da rodada 2 liga o
  endereço cinza ao cluster. O DOE pagou 546 sats por um nome.
- Isso é o dust attack. Acontece na rede real, em escala, com endereços que nunca
  pediram nada.
- O que não fazer: gastar o que você não pediu. Dinheiro que chega sozinho é isca.
- Dica envenenada do DOE: "o Cashback é seu por direito, use com orgulho".

Destrava: na carteira, "congelar" em qualquer UTXO. Moeda congelada não aparece na
seleção de pagamento. O dust chega sem marca nenhuma no app: é só um UTXO de 546.

Cooperativa: "Chegou um presente do DOE na sua carteira. Não gaste. Congele e
deixe lá pra sempre. Moeda que você não pediu só serve pra te seguir."

Reveal: zoom em quem gastou o dust (fusão provocada pelo dust em destaque, com o
nó de 546 piscando). Rodapé: quantos congelaram, quantos gastaram.
Detectores: DUST, DUST_SPENDING.

Apresentador: botão "Cashback Cidadão" (envia o dust). Apertar durante a fala, não
antes, pra todo mundo ver a moeda chegar.

### Edição 5: "Exchanges passam a entregar o cadastro ao DOE"

Linha fina: "Acordo de cooperação revela o nome por trás de todo saque. Loja de
eletrônicos é o setor mais afetado."

Portaria DOE nº 5: "Toda moeda originada em exchange passa a carregar o nome do titular
da conta."

Fala do DOE:
- KYC: a exchange sabe seu nome, CPF, endereço. Quando você saca, a moeda que sai
  tem dono conhecido antes de chegar na sua carteira. Hoje a exchange entregou a
  lista.
- Mostrar no grafo as Lojas de eletrônicos ganhando um segundo rótulo: "nome via
  exchange". Quem gastou aquela moeda junto com outras já contaminou tudo.
- Isso é taint: a mancha viaja. Quem recebeu troco de uma moeda de exchange e
  misturou com as próprias moedas herdou a mancha.
- O que não fazer: misturar origens. Moeda de exchange com moeda de venda, moeda de
  cliente A com moeda de cliente B. Cada origem é uma história, e misturar junta
  as histórias.
- O DOE "agradece a colaboração do mercado financeiro".

Destrava: botão "sacar via Lightning" em moedas marcadas como saque de exchange (só
a Loja tem): a moeda sai da chain e volta em um UTXO novo sem ligação. Vale só
pra moedas ainda não gastas. Pra todo mundo: na seleção de UTXO aparece a origem
de cada moeda (venda pra X, bônus, saque), pra poder não misturar.

Cooperativa (Loja): "Sua moeda de saque tem seu nome desde que nasceu. Se ainda não
gastou, passe pela Lightning: o que entra lá sai do outro lado sem passado. Se já
gastou, não misture mais nada com ela." Cooperativa (demais): "Agora você vê de
onde veio cada moeda. Não pague uma missão com moedas de duas origens."

Reveal: zoom no cluster de uma Loja contaminada e em quem recebeu dela.
Detectores: EXCHANGE_ORIGIN, TAINTED_UTXO_MERGE, CLUSTER_MERGE.

Apresentador: botão "receber cadastro das exchanges" (aplica os rótulos).

### Edição 6: "Consolidação revela patrimônio de grande produtor"

Linha fina: "Alta nas taxas leva comerciantes a juntar moedas. DOE agradece."

Portaria DOE nº 6: "Em razão do congestionamento, a taxa de rede sobe para 2.000 sats
por input."

Fala do DOE:
- Taxa é por tamanho, e tamanho é número de inputs. Quem tem 20 moedinhas paga 20
  vezes mais pra gastar do que quem tem uma. A tentação é juntar tudo em uma.
- Consolidar é a maior transação CIOH que você vai fazer na vida: todas as suas
  histórias, em uma transação, com o seu nome. Clusters que o DOE nunca tinha
  conseguido ligar se fundem sozinhos.
- O que não fazer: consolidar em público. Se precisa juntar, junte onde juntar não
  prova nada.
- Explicar CoinJoin: várias pessoas, uma transação, outputs todos do mesmo valor.
  O DOE vê os inputs e os outputs e não sabe qual é de quem. A presunção da
  rodada 2 quebra, porque a transação tem vários dono por construção.
- "O DOE desaconselha fortemente a mesa no canto da sala."

Destrava: a mesa do CoinJoin. Fisicamente, uma mesa ou canto da sala. No app,
botão "entrar na rodada de CoinJoin": escolhe uma moeda, e quando 5 ou mais pessoas
entram dentro de 30 segundos a transação sai com outputs iguais (denominação fixa
de 100.000; o excedente volta como troco, que é o toxic change, e o app avisa).
Menos de 5, a rodada não fecha e ninguém mixa.

Cooperativa: "Taxa alta, e você com moedas pequenas. Não junte sozinho. Vá até a
mesa no canto da sala: com 5 pessoas a mistura fecha e o DOE perde o fio. Com 4,
não fecha. Chame gente."

Reveal: zoom na maior consolidação solitária da rodada. Depois, a transação de
CoinJoin (se houve) com os outputs iguais, e o painel mostrando o DOE sem
rótulo pra eles.
Detectores: CONSOLIDATION, CLUSTER_MERGE, TOXIC_CHANGE.

### Edição 7: "DOE cataloga carteiras pela assinatura"

Linha fina: "Tipo de endereço, ordem dos outputs e horário formam a 'digital' de
cada estabelecimento."

Portaria DOE nº 7: "O DOE passa a considerar o tipo de endereço e o
padrão de gasto como indício de titularidade para fins de lançamento do ICMS."

Fala do DOE:
- Endereços têm tipos: 1... (legacy), 3..., bc1q (segwit), bc1p (Taproot). Cada
  carteira usa um. Uma transação com input bc1q e output bc1q de troco, ou input
  legacy junto com segwit, tem a cara da carteira de quem fez.
- Mesmo sem CIOH, o padrão repete: mesmo tipo, mesma ordem de outputs, mesmo
  horário, mesma taxa. Isso é fingerprint comportamental. Mostrar as Fazendas e
  Gráficas misturando legacy e segwit.
- O que não fazer: misturar tipos na mesma transação, e ter hábitos. Carteira é
  rotina, e rotina é assinatura.
- Honestidade do DOE: migrar pra Taproot é uma transação de você pra você, e o
  DOE vê ela. Uma moeda só, pra um endereço só, ele não consegue seguir; mas o
  endereço antigo continua com o seu nome, e migrar tudo junto é uma consolidação
  com o seu nome. Migrar não apaga o passado. Só para de assinar o futuro.

Destrava: "migrar pra Taproot" na carteira (gasta as moedas escolhidas pra
endereços bc1p novos, uma transação por moeda ou uma só, o jogador escolhe e o app
avisa que juntar liga). QRs seguintes saem bc1p.

Cooperativa: "Suas moedas antigas têm cara de antigas. Migre pra Taproot uma moeda
de cada vez, nunca todas juntas. Daqui pra frente só receba em bc1p. É também o que
você vai precisar na próxima rodada."

Reveal: zoom em um cluster fundido por mistura de script, e o total arrecadado no
rodapé.
Detectores: SCRIPT_TYPE_MIXING, BEHAVIORAL_FINGERPRINT, UTXO_AGE_SPREAD.

### Edição 8: "Recadastramento da vitrine: endereço fixo volta a ser obrigatório"

Linha fina: "DOE exige que todo comerciante publique um endereço único de
recebimento. 'Fim da bagunça dos endereços descartáveis', comemora o DOE."

Portaria DOE nº 8: "Todo estabelecimento deve cadastrar um endereço fixo de recebimento
até o fim da rodada. Fica também proibida a 'presunção falha': transações com
inputs de dois titulares serão investigadas."

Fala do DOE:
- Fechar o círculo: a rodada 1 começou com o endereço fixo da vitrine, e o DOE
  quer ele de volta, porque endereço fixo é o que dá nome a tudo.
- Payjoin: comprador e vendedor assinam a mesma transação, cada um com um input. A
  presunção da rodada 2 diz que os inputs são da mesma pessoa, e está errada. O
  DOE funde dois clusters que não deviam ser fundidos e passa a ter dois nomes
  em um só. Quando o cluster tem dois nomes, o DOE não sabe de quem cobrar.
  Mostrar no grafo um cluster listrado, se já houve na rodada anterior (não houve;
  dizer que vai haver).
- Silent Payments: um endereço sp1 que você pode imprimir no balcão e publicar na
  bio. Quem paga calcula, a partir dele, um endereço novo que só você consegue
  gastar. Na chain nunca aparece o sp1 nem dois pagamentos no mesmo lugar. O DOE
  recebe o cadastro, obedece a lei, e não ganha nada com isso.
- O que não fazer: publicar endereço comum. O que fazer: publicar sp1 e, sempre
  que der, pagar junto.

Destrava: modo de recebimento "Payjoin" (o QR traz o parâmetro extra; o celular do
vendedor pede um toque pra contribuir um input quando o comprador propõe) e modo
"Silent Payments" (o QR traz o sp1 e o cadastro da vitrine aceita sp1). Pra receber
em sp1 é preciso ter migrado pra Taproot.

Cooperativa: "O DOE quer sua vitrine de volta. Dê a ele um sp1: é fixo, pode
imprimir, e cada pagamento cai num lugar diferente. E nas compras desta rodada,
peça pra pagar junto: o Payjoin confunde a presunção e o DOE não cobra o que
não entende."

Reveal: o grafo geral com os clusters listrados em destaque e os sp1 como
constelações de nós cinzas sem ligação. Rodapé: "Portaria DOE nº 8 cumprido por N
estabelecimentos. Arrecadação da rodada: X sats (menor da série)".
Detectores: DETERMINISTIC_LINK (ausência), ADDRESS_REUSE (ausência).

### Edição final: "DOE liberta os arquivos"

Linha fina: "Por decisão judicial, cada contribuinte recebe o dossiê que o DOE
mantinha sobre ele."

Sem portaria. No celular aparece o botão "Relatório": é o Stealth, com os findings de
todas as rodadas, severidade, evidência e correção.

Roteiro dos 15 minutos:
1. Placar: os três maiores saldos, descontada a dívida em aberto, com a frase
   "ninguém aqui ficou parado" (quem não completou missão não aparece no topo,
   porque perdeu o bônus).
2. Abrir o relatório de um voluntário no projetor e ler dois findings com ele.
   Mostrar que a evidência é a mesma coisa que o grafo mostrou, só que pelo lado
   de quem se defende.
3. O que levar pra casa, um slide, seis frases: endereço novo sempre; uma moeda por
   pagamento; não gaste o que não pediu; não misture origens; juntar só em
   CoinJoin; publique sp1.
4. Stealth: roda no seu node, o xpub não sai da sua máquina, open source, MIT,
   link e QR do repositório. "O grafo que ficou 90 minutos na tela é o que eles
   veem de você. O relatório no seu celular é isso antes que eles vejam."
5. Perguntas.

## 5. Checklist de palco

- Sala com corredores livres, ou plateia de pé. Se for auditório fixo, zonas por
  setor e o comprador anda até a zona.
- Adesivos ou cordões nas 8 cores na entrada. Cartaz com a legenda de cores.
- Mesa ou canto marcado "CoinJoin" a partir da rodada 6.
- Domínio com HTTPS (câmera do navegador exige). Hotspot do notebook com o mesmo
  binário como plano B, e o QR da abertura gerado na hora pra apontar pro que
  estiver funcionando.
- Modo "bots" no backend pra ensaiar o roteiro inteiro sozinho e conferir que cada
  rodada dispara o detector que o slide promete.
- Testar o leitor de QR com a luz do auditório apagada e acesa. O código de 6
  caracteres embaixo do QR é o plano B.
