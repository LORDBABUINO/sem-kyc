# Trilhas de crescimento, 2 compras por rodada, 8 rodadas. Sigla do fornecedor entre parênteses.
# P Padaria, F Fazenda, E Energia, T Transportadora, L Loja eletr., C Construtora, K Consultoria, G Gráfica
paths = {
"P":[("farinha e ovos","F"),("forno elétrico","L"),("balcão e reforma","C"),("alvará sanitário","K"),
     ("cardápios e letreiro","G"),("painel solar do forno","E"),("entregas em domicílio","T"),("lote grande de farinha","F"),
     ("embalagens de marca","G"),("sistema de pedidos","L"),("distribuição pra mercados","T"),("contrato de franquia","K"),
     ("segunda loja","C"),("ampliação da energia","E"),("forno da segunda loja","L"),("farinha da segunda loja","F")],
"F":[("frete da colheita","T"),("licença ambiental","K"),("bomba de irrigação","E"),("galpão de estoque","C"),
     ("sensores de solo","L"),("embalagens e rótulos","G"),("refeições da colheita","P"),("frete da safra","T"),
     ("energia do galpão","E"),("certificação orgânica","K"),("catálogo de produtos","G"),("drone de pulverização","L"),
     ("silo","C"),("refeições da equipe","P"),("frete da safra grande","T"),("energia do silo","E")],
"E":[("inversores","L"),("frete dos painéis","T"),("estrutura no telhado","C"),("contrato com a concessionária","K"),
     ("arrendamento de terreno","F"),("material de venda","G"),("refeições da equipe de instalação","P"),("baterias","L"),
     ("frete da ampliação","T"),("subestação","C"),("certificação técnica","K"),("catálogo técnico","G"),
     ("expansão do terreno","F"),("refeições da equipe","P"),("inversores novos","L"),("frete da usina","T")],
"T":[("recarga da frota","E"),("seguro e licença","K"),("rastreadores","L"),("garagem","C"),
     ("adesivagem da frota","G"),("lanches de bordo","P"),("contrato de escoamento","F"),("recarga da frota","E"),
     ("sistema de rotas","L"),("renovação do seguro","K"),("ampliação da garagem","C"),("adesivagem da frota nova","G"),
     ("lanches da equipe","P"),("contrato da safra","F"),("recarga da frota ampliada","E"),("rastreadores novos","L")],
"L":[("fachada e panfletos","G"),("contrato de importação","K"),("frete da mercadoria","T"),("vitrine e reforma","C"),
     ("painel solar","E"),("coffee break de lançamento","P"),("cestas pros funcionários","F"),("catálogo","G"),
     ("frete do novo lote","T"),("regularização fiscal","K"),("segunda loja","C"),("energia da segunda loja","E"),
     ("coffee break da segunda loja","P"),("cestas de fim de ano","F"),("frete da segunda loja","T"),("catálogo novo","G")],
"C":[("alvará de obras","K"),("frete de material","T"),("ferramentas elétricas","L"),("energia do canteiro","E"),
     ("refeições do canteiro","P"),("placas de obra","G"),("madeira e terra","F"),("regularização","K"),
     ("frete da obra grande","T"),("drone de inspeção","L"),("energia do canteiro novo","E"),("refeições da obra grande","P"),
     ("placas da obra nova","G"),("madeira da obra grande","F"),("frete final","T"),("ferramentas novas","L")],
"K":[("reforma do escritório","C"),("computadores","L"),("cartões e material impresso","G"),("energia do escritório","E"),
     ("coffee break de clientes","P"),("retiro da equipe na fazenda","F"),("mudança pra sede nova","T"),("sede nova","C"),
     ("servidor","L"),("relatório anual impresso","G"),("energia da sede nova","E"),("coffee break do evento","P"),
     ("evento de clientes no campo","F"),("transporte do evento","T"),("computadores novos","L"),("material do evento","G")],
"G":[("impressora","L"),("frete do papel","T"),("energia das máquinas","E"),("contrato de fornecimento","K"),
     ("ampliação do galpão","C"),("refeições da equipe","P"),("fibra pro papel artesanal","F"),("frete do papel grande","T"),
     ("impressora de grande formato","L"),("registro de marca","K"),("energia da máquina nova","E"),("refeições do turno da noite","P"),
     ("galpão novo","C"),("fibra do lote grande","F"),("frete do lote grande","T"),("impressora nova","L")],
}
S=list(paths); names={"P":"Padaria","F":"Fazenda","E":"Energia","T":"Transp.","L":"Loja","C":"Constr.","K":"Consult.","G":"Gráfica"}
for s,p in paths.items(): assert len(p)==16 and all(x[1]!=s for x in p), s
print("| Rod. | "+" | ".join(names[s] for s in S)+" |"); print("|---|"+"---|"*8)
for r in range(8):
    print(f"| {r+1} | "+" | ".join(f"{paths[s][2*r][0]} ({paths[s][2*r][1]}), {paths[s][2*r+1][0]} ({paths[s][2*r+1][1]})" for s in S)+" |")
print()
print("Compradores por vendedor e rodada (setores):")
for r in range(8):
    cnt={s:0 for s in S}
    for s in S:
        for k in (2*r,2*r+1): cnt[paths[s][k][1]]+=1
    print(r+1, "  ".join(f"{names[k]} {v}" for k,v in cnt.items()), "  zeros:", [names[k] for k,v in cnt.items() if v==0])
