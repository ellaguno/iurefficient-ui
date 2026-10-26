# Atribución de activos

Los iconos se dibujan en código (`src/icons.rs`); los únicos activos de terceros son las fuentes.

| Archivo | Autor | Licencia | Fuente |
|---|---|---|---|
| `assets/fonts/Inter-Regular.ttf`, `Inter-Medium.ttf`, `Inter-SemiBold.ttf`, `Inter-Bold.ttf` | The Inter Project Authors (Rasmus Andersson) | SIL Open Font License 1.1 ([`assets/fonts/OFL.txt`](assets/fonts/OFL.txt)) | Inter 4.1, `Inter-4.1.zip` → `extras/ttf/`, <https://github.com/rsms/inter/releases/tag/v4.1> (sin cambios) |
| `assets/fonts/InterTabular-Regular.ttf` | Derivada de Inter-Regular.ttf 4.1 (The Inter Project Authors) | SIL Open Font License 1.1 ([`assets/fonts/OFL.txt`](assets/fonts/OFL.txt)) | Generada con [`assets/fonts/inter-tabular.py`](assets/fonts/inter-tabular.py) (fontTools): `tnum` congelado en el cmap, subconjunto de cifras y puntuación, familia renombrada «Inter Tabular». Inter no tiene nombres reservados. |

Las fuentes se incluyen en el binario solo con la característica `bundled-fonts` (activa por
omisión). La OFL permite incluirlas y redistribuirlas con cualquier programa siempre que viaje el
texto de la licencia (`assets/fonts/OFL.txt`) y que no se vendan solas; las apps que las
distribuyan deben copiar ese archivo junto a sus propios avisos de terceros.
