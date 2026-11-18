# API do servidor

Contrato entre `fiscal-server` e o frontend. JSON em tudo; valores em sats
(inteiros); endereços como string; `outpoint` é `"txid:vout"`.

## Autenticação

- Jogador: header `X-Session: <token>` devolvido pelo `join`. O frontend guarda em
  `localStorage` e manda em toda chamada e no WebSocket (`/ws?session=<token>`).
- Apresentador: header `X-Admin: <token>` (token vem da variável `FISCAL_ADMIN_TOKEN`
  ou da flag `--admin-token`). WebSocket em `/ws/admin?token=<token>`.
- Projetor: sem autenticação. WebSocket em `/ws/projector`.

Erros: status 4xx com `{"error": "<mensagem em português pra tela>"}`.

## Tipos

```
Phase      = {phase:"lobby"} | {phase:"newspaper", round} | {phase:"negotiation", round, deadline}
           | {phase:"reveal", round} | {phase:"final"}
Sector     = "padaria"|"fazenda"|"energia"|"transportadora"|"loja"|"construtora"|"consultoria"|"grafica"
SectorInfo = {sector, name, color}             // name em pt-BR, color "#rrggbb"
Coin       = {outpoint, address, amount, kind:"legacy"|"segwit"|"taproot", frozen,
              origin:{kind:"initial"|"exchange_withdrawal"|"sale"|"change"|"bonus"|"dust"|"lightning"|"coin_join"|"migration"|"tax", from?},
              round}
Item       = {name, supplier: Sector}
Mission    = {round, text, items:[{item, status:{status:"pending"|"paid"|"expired", txid?}, price, supplier: SectorInfo}], bonus: txid|null, complete}
Invoice    = {code, seller, seller_name, seller_sector: SectorInfo, item, amount, display, uri, payjoin, round,
              status:{status:"open"|"payjoin_pending"|"paid"|"expired", buyer?, txid?}}
             // uri = "bitcoin:<display>?amount=<btc>&label=<item>[&pj=<url>]"
HistoryEntry = {txid, direction:"in"|"out"|"internal", amount, counterparty: string|null, memo, height: int|null, round}
Auto       = {id, player, name, round, amount, paid, late_rounds}
Unlock     = "fresh_address"|"coin_control"|"odd_price"|"freeze"|"lightning"|"coin_join"|"taproot"|"payjoin"|"silent_payments"|"report"
Identity   = {identity:"unknown"} | {identity:"known", players:[id]} | {identity:"conflict", players:[id,...]}
```

## Jogador

`POST /api/join` `{name}` → `{session, me: Me}`. Erros: nome vazio, nome repetido.

`GET /api/me` → `Me`:
```
{
  player: {id, name, sector: SectorInfo, storefront, registered, storefront_is_silent,
           receive_mode:"storefront"|"fresh"|"payjoin"|"silent_payments",
           price_style:"round"|"odd", coin_selection:"automatic"|"smallest_covering"|"manual",
           unlocked:[Unlock], taproot, silent_address: string|null},
  phase: Phase, round, height,
  balance, coins:[Coin],
  mission: Mission|null,            // da rodada atual
  missions:[Mission],
  catalog:[{name, price}],          // o que eu vendo, preço desta rodada (0 fora da negociação)
  invoices:[Invoice],               // os meus, da rodada atual
  payjoin_requests:[Invoice],       // pedidos meus esperando minha resposta
  history:[HistoryEntry],
  autos:[Auto], suspended: bool, debt,
  counter_address,                  // endereço do guichê do ICMS
  coinjoin: {entries, deadline}|null,
  report_available: bool
}
```

`POST /api/register` → `{txid}` (paga a taxa de cadastro; primeira transação).

`POST /api/settings` `{receive_mode?, price_style?, coin_selection?}` → `Me`.

`POST /api/coins/freeze` `{outpoint, frozen}` → `Me`.

`POST /api/invoices` `{item}` → `Invoice` (eu sou o vendedor).

`GET /api/invoices/{code}` → `Invoice` (qualquer jogador; é o que o comprador vê ao escanear).

`POST /api/invoices/{code}/pay` `{inputs?:[outpoint], payjoin?: bool}` →
`{result:"settled", txid}` ou `{result:"awaiting_seller"}`.

`POST /api/invoices/{code}/payjoin` `{contribute: outpoint|null}` → `{txid}` (vendedor responde).

`POST /api/lightning` `{outpoint}` → `{out, inbound}`.

`POST /api/coinjoin` `{outpoint}` → `{entries, deadline}`.

`POST /api/taproot` `{outpoints:[outpoint], together: bool}` → `{txids}`.

`POST /api/storefront/silent` → `{sp}` (recadastra a vitrine como sp1).

`POST /api/autos/{id}/pay` `{inputs?:[outpoint]}` → `{txid}`.

`GET /api/report` → relatório do Stealth (`{stats, findings:[{type, severity, description, details, correction}], warnings, summary}`); 403 antes da edição final.

## Público (projetor)

`GET /api/state` →
```
{
  phase, round, height, players: int, registered: int,
  counter_address, join_url,
  reveal: Reveal|null,              // da última rodada fechada
  autos_open:[Auto],                // dívida ativa
  totals: {assessed, collected},
  payments_this_round: int,
  standings:[{player, name, balance}] // só na fase final
}
Reveal = {round, headline:{round, player, finding}|null, headline_name, autos:[Auto], assessed, collected, payments,
          known:{attributed_operations, real_operations, linked_addresses, counterparties:[name]}}
```

`GET /api/clusters` →
```
{clusters:[{root, addresses:[string], identity: Identity, names:[string], color: string|null, balance}],
 edges:[{txid, from: root, to: root, amount}]}
```

## Apresentador (`X-Admin`)

- `POST /api/admin/advance` → `{phase}`
- `POST /api/admin/open` → `{phase}`
- `POST /api/admin/close` → `{phase, reveal}`
- `POST /api/admin/dust` → `{txid}`
- `POST /api/admin/kyc` → `{players:[id]}`
- `POST /api/admin/mine` → `{height}`
- `POST /api/admin/autos/{id}/forgive` → `{}`
- `POST /api/admin/bots` `{count}` → `{players:[id]}` (preenche setores e joga sozinho)
- `POST /api/admin/reset` → `{}`
- `GET /api/admin/room` → `{state: (como /api/state), players:[{id, name, sector, registered, balance, suspended, mission_complete}], invoices_open: int, coinjoin, log:[HistoryEntry-like]}`
- `POST /api/admin/config` `{negotiation_secs?, icms_percent?, penalty_percent?, late_percent?, target_revenue?}` → config

## WebSocket

Mensagens JSON `{event, ...}`. Todo cliente recebe `{event:"phase", phase}` ao conectar.

Jogador (`/ws?session=`): `phase`, `mission` (nova rodada: `{round}`), `unlock` `{unlock}`,
`invoice_paid` `{code, txid}`, `payjoin_request` `{code}`, `tx` `{txid}`, `auto` `{auto_id, amount, paid, txid?}`,
`bonus` `{txid, amount}`, `report_available`, `coinjoin` `{txid|null}`. Ao receber qualquer
um, o celular refaz `GET /api/me`.

Projetor (`/ws/projector`): `phase`, `block` `{height}`, `tx` `{txid, clusters:[ClusterEvent]}` com
`ClusterEvent = {event:"merged", absorbed, into, cause:"common_input"|"change", txid}
| {event:"labeled", root, player} | {event:"conflict", root, players}`,
`kyc` `{clusters:[ClusterEvent]}`, `reveal` `{round}`, `auto` `{auto_id, player, amount, paid}`,
`coinjoin` `{txid|null, players}`, `payments` `{round, count}`. Ao receber `tx`, o projetor
busca `GET /api/clusters` se quiser o estado completo, ou aplica os eventos.

Apresentador (`/ws/admin?token=`): tudo acima mais `player_joined` `{player}` e `invoice_paid` com nomes.
