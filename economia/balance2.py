import random, re
src=open("demanda2.py").read().split("S=list(paths)")[0]
exec(src)
S=list(paths); names={"P":"Padaria","F":"Fazenda","E":"Energia","T":"Transp.","L":"Loja","C":"Constr.","K":"Consult.","G":"Gráfica"}
random.seed(3)
orig={s:list(p) for s,p in paths.items()}
def ok(s,seq):
    # ordem relativa por fornecedor preservada; 4 primeiros originais nas rodadas 1-3; 4 últimos nas rodadas 6-8
    pos={id(x):i for i,x in enumerate(seq)}
    for sup in S:
        idx=[pos[id(x)] for x in orig[s] if x[1]==sup]
        if idx!=sorted(idx): return False
    for x in orig[s][:4]:
        if pos[id(x)]>5: return False
    for x in orig[s][-4:]:
        if pos[id(x)]<10: return False
    return True
def cost(cur):
    c=0
    for r in range(8):
        cnt={k:0 for k in S}
        for s in S:
            for k in (2*r,2*r+1): cnt[cur[s][k][1]]+=1
        c+=sum((v-2)**2 for v in cnt.values())
    return c
cur={s:list(p) for s,p in paths.items()}; cc=cost(cur)
for it in range(400000):
    s=random.choice(S); i,j=random.sample(range(16),2)
    seq=cur[s][:]; seq[i],seq[j]=seq[j],seq[i]
    if not ok(s,seq): continue
    old=cur[s]; cur[s]=seq; nc=cost(cur)
    if nc<=cc: cc=nc
    else: cur[s]=old
    if cc==0: break
print("custo",cc)
print("| Rod. | "+" | ".join(names[s] for s in S)+" |"); print("|---|"+"---|"*8)
for r in range(8):
    print(f"| {r+1} | "+" | ".join(f"{cur[s][2*r][0]} ({cur[s][2*r][1]}), {cur[s][2*r+1][0]} ({cur[s][2*r+1][1]})" for s in S)+" |")
print()
for r in range(8):
    cnt={k:0 for k in S}
    for s in S:
        for k in (2*r,2*r+1): cnt[cur[s][k][1]]+=1
    print(r+1, "  ".join(f"{names[k]} {v}" for k,v in cnt.items()))
import json; json.dump({s:[list(x) for x in cur[s]] for s in S}, open("trilhas.json","w"), ensure_ascii=False, indent=1)
