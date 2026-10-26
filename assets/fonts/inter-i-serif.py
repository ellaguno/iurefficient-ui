"""Congela `cv08` («I» mayúscula con remates) de Inter en el cmap, solo para la I latina y sus
formas acentuadas (I, Í, Ì, Î, Ï, Ĩ, Ī, Ĭ, Į, İ, Ǐ, Ȉ, Ȋ, Ỉ, Ị, Ḭ, Ḯ, Ɨ). La iota griega y la І
cirílica, que `cv08` también cambia, se quedan como están; todo lo demás queda sin tocar. egui
0.37 no deja activar funciones OpenType, así que la variante se hornea en el archivo.

Inter no declara nombres reservados (OFL-1.1 sin «Reserved Font Name»), pero la versión
modificada se renombra a «Inter Iure» y lo anota en la versión y la descripción.

Uso (fontTools 4.x), con los TTF de `Inter-4.1.zip` → `extras/ttf/`:
    python3 -I inter-i-serif.py Inter-Regular.ttf Inter-Regular.ttf
"""
import sys
from fontTools.ttLib import TTFont

src, dst = sys.argv[1], sys.argv[2]
f = TTFont(src)
gsub = f['GSUB'].table
idx = {i for fr in gsub.FeatureList.FeatureRecord if fr.FeatureTag == 'cv08' for i in fr.Feature.LookupListIndex}
mapping = {}
for i in sorted(idx):
    lk = gsub.LookupList.Lookup[i]
    for st in lk.SubTable:
        if lk.LookupType == 7:
            st = st.ExtSubTable
        if hasattr(st, 'mapping'):
            mapping.update(st.mapping)
assert mapping.get('I') == 'I.1', 'cv08 ya no es la «I» con remates: revisar la versión de Inter'


def latin(cp):
    return cp <= 0x024F or 0x1E00 <= cp <= 0x1EFF


changed = set()
for t in f['cmap'].tables:
    for cp, g in list(t.cmap.items()):
        if g in mapping and latin(cp):
            t.cmap[cp] = mapping[g]
            changed.add(cp)

style = f['name'].getDebugName(2) or 'Regular'
version = f['name'].getDebugName(5) or ''
note = 'Modificada para Iurefficient: cv08 (I con remates) congelado en el cmap'
for rec in f['name'].names:
    if rec.nameID in (1, 16):
        rec.string = 'Inter Iure'
    elif rec.nameID == 4:
        rec.string = 'Inter Iure ' + style
    elif rec.nameID == 6:
        rec.string = 'InterIure-' + style.replace(' ', '')
    elif rec.nameID == 3:
        rec.string = version + ';InterIure-' + style.replace(' ', '')
    elif rec.nameID == 5:
        rec.string = version + '; ' + note
f['name'].setName(note + '. Original: Inter 4.1, https://github.com/rsms/inter', 10, 3, 1, 0x409)
f.save(dst)
print(dst, 'I con remates en', ''.join(chr(c) for c in sorted(changed)))
