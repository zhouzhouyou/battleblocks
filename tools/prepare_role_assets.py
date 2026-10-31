"""Import generated role frames onto the game's fixed pixel grid.

Creative drawing remains in versioned ImageGen sources. This deterministic import
only slices, anchors, nearest-neighbor samples, applies a fixed palette, and packs.
"""
from pathlib import Path
from PIL import Image
import json
from collections import deque

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'assets/roles'; OUT.mkdir(exist_ok=True)
FRAME=384
DENSITY=4
SPECS={
 'knight': {'rows':[0,320,620,932,1254], 'body_heights':[231,281,248,266], 'height':64,
 'palette':['161820','373847','787080','E2D4BC','FFF0CD','653341','9F4350','D87066','583C31','BC8552','DAB573','885739','45658A','7699B7']},
 'dwarf': {'rows':[0,346,626,940,1254], 'body_heights':[270,266,240,240], 'height':32,
 'palette':['161820','373847','787080','E2D4BC','653341','9F4350','D87066','583C31','885739','BC8552','DAB573','FFF0CD','E66B27','F29A45']},
 'priest': {'rows':[0,322,638,934,1254], 'body_heights':[302,296,284,276], 'height':64,
 'palette':['161820','373847','535064','E2D4BC','FFF0CD','352736','9F4350','D87066','583C31','885739','BC8552','DAB573','293D32','426644']},
}
def clean_alpha(alpha):
 # Discard tiny disconnected fragments that came from adjacent generated cells.
 w,h=alpha.size; data=bytearray(alpha.tobytes()); seen=bytearray(w*h); components=[]
 for start,value in enumerate(data):
  if not value or seen[start]: continue
  component=[]; queue=deque([start]); seen[start]=1
  while queue:
   at=queue.popleft(); component.append(at); x,y=at%w,at//w
   for ny in range(max(0,y-1),min(h,y+2)):
    for nx in range(max(0,x-1),min(w,x+2)):
     p=ny*w+nx
     if data[p] and not seen[p]: seen[p]=1; queue.append(p)
  components.append(component)
 largest=max(map(len,components),default=0); out=bytearray(w*h)
 for component in components:
  if len(component)>=largest*.03:
   for p in component: out[p]=255
 return Image.frombytes('L',(w,h),bytes(out))

report={}
for name,spec in SPECS.items():
 directory=ROOT/'asset-sources/art-direction/characters'/name/'production/v3'
 source=Image.open(directory/'animation-source.png').convert('RGBA')
 assert source.size==(1254,1254), (name,source.size)
 opaque=source.getchannel('A').point(lambda a:255 if a>=160 else 0)
 palette=Image.composite(source.convert('RGB'),Image.new('RGB',source.size,(22,24,32)),opaque).quantize(colors=64,method=Image.Quantize.MEDIANCUT,dither=Image.Dither.NONE)
 rgb=[tuple(palette.getpalette()[i:i+3]) for i in range(0,192,3)]
 spec['palette']=[''.join(f'{v:02X}' for v in color) for color in rgb]
 frames=[]; checks=[]; row_heights=[]
 for row in range(4):
  heights=[]
  for col in range(4):
   a=source.getchannel('A').crop((round(col*source.width/4),spec['rows'][row],round((col+1)*source.width/4),spec['rows'][row+1]))
   box=clean_alpha(a.point(lambda v:255 if v>=160 else 0)).getbbox()
   if box: heights.append(box[3]-box[1])
  row_heights.append(max(heights))
 for row in range(4):
  for col in range(4):
   bounds=(round(col*source.width/4),spec['rows'][row],round((col+1)*source.width/4),spec['rows'][row+1])
   cell=source.crop(bounds)
   alpha=clean_alpha(cell.getchannel('A').point(lambda a:255 if a>=160 else 0))
   bbox=alpha.getbbox(); assert bbox, (name,row,col,'empty')
   cell.putalpha(alpha); cell=cell.crop(bbox)
   scale=320/row_heights[row]
   size=(max(1,round(cell.width*scale)),max(1,round(cell.height*scale)))
   assert size[0]<=FRAME-4 and size[1]<=84*DENSITY, (name,row,col,size)
   cell=cell.resize(size,Image.Resampling.NEAREST)
   colors=cell.convert('RGB').quantize(palette=palette,dither=Image.Dither.NONE).convert('RGBA')
   colors.putalpha(cell.getchannel('A'))
   frame=Image.new('RGBA',(FRAME,FRAME)); frame.alpha_composite(colors,((FRAME-size[0])//2,84*DENSITY-size[1]))
   # Canonical hidden RGB avoids meaningless decoder/palette differences.
   frame.putdata([px if px[3] else (0,0,0,0) for px in frame.getdata()])
   visible={p[:3] for p in frame.getdata() if p[3]}
   assert len(visible)<=64
   assert set(frame.getchannel('A').getdata())<={0,255}
   frames.append(frame); checks.append({'row':row,'column':col,'colors':len(visible),'bounds':frame.getbbox()})
 atlas=Image.new('RGBA',(4*FRAME,4*FRAME))
 montage=Image.new('RGBA',(4*FRAME,4*FRAME))
 for i,frame in enumerate(frames):
  atlas.alpha_composite(frame,((i%4)*FRAME,(i//4)*FRAME)); montage.alpha_composite(frame,((i%4)*FRAME,(i//4)*FRAME))
 frames[0].save(directory/'normal-idle-0.png')
 frames[4].save(directory/'awake-idle-0.png')
 atlas.save(OUT/f'{name}.webp',lossless=True,exact=True,method=6)
 montage.save(directory/'grid-384.png')
 # Inspectable enlarged derivative, never used as the in-game source.
 montage.resize((1536,1536),Image.Resampling.NEAREST).save(directory/'grid-preview.png')
 for phase,row in [('normal-idle',0),('awake-idle',1),('normal-cast',2),('awake-cast',3)]:
  seq=[f.resize((384,384),Image.Resampling.NEAREST) for f in frames[row*4:row*4+4]]
  seq[0].save(directory/f'{phase}.gif',save_all=True,append_images=seq[1:],duration=180 if row<2 else 130,loop=0,disposal=2,transparency=0)
 report[name]={'frame_size':[384,384],'frames':16,'visible_height':320,'display_scale':'uniform across all three roles','foot_anchor':[192,336],'palette':spec['palette'],'checks':checks,'webp_bytes':(OUT/f'{name}.webp').stat().st_size}

# The book reuses only the existing v6 cover/pages/bottom layers from the v5 model.
bookdir=ROOT/'asset-sources/art-direction/characters/dwarf/animations/v6'
cover=Image.open(bookdir/'book-cover.png').convert('RGBA').resize((160,24),Image.Resampling.NEAREST)
bottom=Image.open(bookdir/'book-bottom.png').convert('RGBA').resize((160,8),Image.Resampling.NEAREST)
book=Image.new('RGBA',(384*4,384))
for level in range(1,4):
 pages=Image.open(bookdir/'book-pages.png').convert('RGBA').resize((160,level*12),Image.Resampling.NEAREST)
 height=(8+level*3)*4
 book.alpha_composite(cover,(384*level+112,336-height))
 book.alpha_composite(pages,(384*level+112,336-height+24))
 book.alpha_composite(bottom,(384*level+112,328))
book_palette=Image.new('P',(1,1)); book_rgb=[tuple(bytes.fromhex(h)) for h in SPECS['dwarf']['palette']]
book_palette.putpalette([v for c in (book_rgb+[book_rgb[-1]]*(256-len(book_rgb))) for v in c])
def book_colors(im):
 colored=im.convert('RGB').quantize(palette=book_palette,dither=Image.Dither.NONE).convert('RGBA'); colored.putalpha(im.getchannel('A')); return colored
book_colors(book).save(OUT/'grudge-book.webp',lossless=True,exact=True,method=6)
for name,layer in [('cover',cover),('pages',Image.open(bookdir/'book-pages.png').convert('RGBA').resize((160,12),Image.Resampling.NEAREST)),('bottom',bottom)]:
 book_colors(layer).save(OUT/f'book-{name}.webp',lossless=True,exact=True,method=6)
report['book']={'frame_size':[384,384],'frames':4,'heights':[0,11,14,17],'source':'dwarf/animations/v6/'}
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n')

# Generate a registry including the three new roles. Keep legacy entries for
# diagnostics; the public roster is declared separately in content::ROSTER.
legacy=json.loads((ROOT/'assets/report.json').read_text())
normal=[]; awakened=[]; thumbs=Image.new('RGBA',(440,48))
for i,name in enumerate(['knight','elf','wizzard','goblin','imp','chort','big_zombie']):
 meta=legacy[name]; w,h=meta['frame_size']
 normal.append(f'    SpriteAsset {{bytes:include_bytes!("../assets/{name}.webp"),w:{w}.,h:{h}.,durations:&{meta["durations"]},indices:&{meta["indices"]}}},')
 frame=Image.open(ROOT/f'assets/{name}.webp').crop((0,0,w,h)); frame.thumbnail((40,48),Image.Resampling.NEAREST)
 thumbs.alpha_composite(frame,(i*40+(40-frame.width)//2,48-frame.height))
for name in ['knight','elf','wizard','goblin','imp','chort','zombie']:
 path=ROOT/f'assets/awakened/{name}-awakening.webp'; w,h=Image.open(path).size
 awakened.append(f'    SpriteAsset {{bytes:include_bytes!("../assets/awakened/{name}-awakening.webp"),w:{w}.,h:{h}.,durations:&[1.],indices:&[0]}},')
for i,name in enumerate(['knight','dwarf','priest','nun'],7):
 for entries,indices in [(normal,'[0,1,2,3]'),(awakened,'[4,5,6,7]')]:
  entries.append(f'    SpriteAsset {{bytes:include_bytes!("../assets/roles/{name}.webp"),w:384.,h:384.,durations:&[0.18;4],indices:&{indices}}},')
 frame=Image.open(OUT/f'{name}.webp').crop((0,0,384,384)).resize((48,48),Image.Resampling.NEAREST)
 thumbs.alpha_composite(frame, (i*40-4,0))
thumbs.save(ROOT/'assets/hero-thumbnails.webp',lossless=True,exact=True,method=6)
(ROOT/'src/sprites.rs').write_text('// Generated by tools/prepare_assets.py and tools/prepare_role_assets.py\nuse super::SpriteAsset;\npub const SPRITES: &[SpriteAsset]=&[\n'+'\n'.join(normal)+'\n];\npub const AWAKENED:&[SpriteAsset]=&[\n'+'\n'.join(awakened)+'\n];\n#[cfg(test)]\npub const BOOK_BYTES:&[u8]=include_bytes!("../assets/roles/grudge-book.webp");\npub const BOOK_COVER:&[u8]=include_bytes!("../assets/roles/book-cover.webp");\npub const BOOK_PAGES:&[u8]=include_bytes!("../assets/roles/book-pages.webp");\npub const BOOK_BOTTOM:&[u8]=include_bytes!("../assets/roles/book-bottom.webp");\n')
print(json.dumps({name:{k:v for k,v in meta.items() if k!='checks'} for name,meta in report.items()},ensure_ascii=False))
