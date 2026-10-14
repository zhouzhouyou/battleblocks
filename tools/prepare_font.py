from pathlib import Path
from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
import sys
ROOT=Path(__file__).resolve().parents[1]
source=Path(sys.argv[1])
text=''.join(p.read_text() for p in sorted((ROOT/'src').rglob('*.rs')))
characters=set(chr(c) for c in range(32,127))|{c for c in text if ord(c)>127}
font=TTFont(source)
if "fvar" in font: font=instantiateVariableFont(font,{"wght":600},inplace=True)
missing=characters-{chr(c) for c in font.getBestCmap()}
assert not missing, f'Missing glyphs: {sorted(missing)}'
options=subset.Options();options.name_IDs=['*'];options.name_languages=['*']
subsetter=subset.Subsetter(options=options);subsetter.populate(text=''.join(sorted(characters)));subsetter.subset(font)
output=Path(sys.argv[2]) if len(sys.argv)>2 else ROOT/'assets/ui-subset.ttf'
font.save(output)
print(f'Font subset: {source.stat().st_size} -> {output.stat().st_size} bytes, {len(characters)} characters')
