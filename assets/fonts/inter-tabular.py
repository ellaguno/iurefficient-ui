"""Genera InterTabular-Regular.ttf: congela `tnum` en el cmap de Inter-Regular.ttf y deja solo
cifras y la puntuación de horas, fechas y cantidades (~10 KB). egui 0.37 no deja activar
funciones OpenType, así que las cifras tabulares van en una fuente aparte.

Uso (fontTools 4.x): python3 -I inter-tabular.py Inter-Regular.ttf InterTabular-Regular.ttf "Inter Tabular"
"""
import sys
from fontTools.ttLib import TTFont
from fontTools import subset

src, dst, family = sys.argv[1], sys.argv[2], sys.argv[3]
f = TTFont(src)
gsub = f['GSUB'].table
idx = {i for fr in gsub.FeatureList.FeatureRecord if fr.FeatureTag == 'tnum' for i in fr.Feature.LookupListIndex}
mapping = {}
for i in sorted(idx):
    lk = gsub.LookupList.Lookup[i]
    for st in lk.SubTable:
        if lk.LookupType == 7:
            st = st.ExtSubTable
        if hasattr(st, 'mapping'):
            mapping.update(st.mapping)
chars = "0123456789:/.,-+−%$#    ()–"
changed = 0
for t in f['cmap'].tables:
    for cp, g in list(t.cmap.items()):
        if g in mapping and chr(cp) in chars:
            t.cmap[cp] = mapping[g]; changed += 1
opts = subset.Options()
opts.layout_features = ['kern']  # sin `pnum`/`calt` que deshagan el congelado
opts.name_IDs = ['*']; opts.notdef_outline = True; opts.hinting = True
opts.drop_tables += ['GSUB']
ss = subset.Subsetter(opts)
ss.populate(unicodes=[ord(c) for c in chars])
ss.subset(f)
name = f['name']
for rec in name.names:
    if rec.nameID in (1, 16):
        rec.string = family
    elif rec.nameID == 4:
        rec.string = family + ' Regular'
    elif rec.nameID == 6:
        rec.string = family.replace(' ', '') + '-Regular'
f.save(dst)
g = TTFont(dst); cm = g.getBestCmap()
print(dst, 'remapped', changed, 'digit widths', {g['hmtx'][cm[ord(c)]][0] for c in '0123456789'})
