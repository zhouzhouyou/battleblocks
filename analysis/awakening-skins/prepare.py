"""Prepare reviewed static skin concepts; does not modify game assets or Rust code."""
from pathlib import Path
from PIL import Image
import json
ROOT=Path(__file__).resolve().parent
GENERATED=Path('/Users/yuri/.codex/generated_images/01a178ad-c46d-7c03-b9a0-9ba20ca3030b')
SOURCES=[
('knight','骑士','长剑','exec-db1e7055-4c74-4cd5-a754-be89eb7ab399.png'),
('elf','精灵','短弓','exec-f75b8977-8462-491f-834e-b34b7e2b51d6.png'),
('wizard','巫师','法杖','exec-ca27af11-f45b-4895-af8a-cee8f5a36cdc.png'),
('goblin','哥布林','匕首','exec-2792c3ac-d4f5-4329-88ad-a5709fdf3e5b.png'),
('imp','小恶魔','三叉戟','exec-6b290273-8595-4ab6-a980-bc0fe27fdba6.png'),
('chort','角魔','战斧','exec-1b427d77-a1df-42f6-81b0-caf3a4a7f579.png'),
('zombie','大僵尸','石锤','exec-73f012e7-190a-4671-8d92-111c43d3ef3b.png')]
rows=[]
for key,name,weapon,filename in SOURCES:
 source=GENERATED/filename
 image=Image.open(source).convert('RGBA')
 bbox=image.getchannel('A').getbbox()
 assert bbox,'empty image'
 # Asset preparation only: remove empty margins and sample to a 96px sprite.
 cropped=image.crop(bbox)
 native=cropped.resize((max(1,round(cropped.width*96/cropped.height)),96),Image.Resampling.NEAREST)
 png=ROOT/(key+'-awakening.png');webp=ROOT/(key+'-awakening.webp')
 native.save(png,optimize=True,compress_level=9)
 native.save(webp,format='WEBP',lossless=True,exact=True,method=6)
 decoded=Image.open(webp).convert('RGBA')
 assert decoded.size==native.size and decoded.tobytes()==native.tobytes(),key+' webp mismatch'
 assert Image.open(png).convert('RGBA').tobytes()==native.tobytes()
 rows.append(dict(id=key,name=name,weapon=weapon,source=str(source),source_bytes=source.stat().st_size,png=str(png),webp=str(webp),png_bytes=png.stat().st_size,webp_bytes=webp.stat().st_size,width=native.width,height=native.height,rgba_equal=True,gender='female' if key=='elf' else None,clothing='cloth, no armor' if key=='goblin' else None))
report=dict(status='static review concepts, not integrated into game',format='RGBA PNG and lossless WebP',sprite_height=96,skins=rows,total_png_bytes=sum(r['png_bytes'] for r in rows),total_webp_bytes=sum(r['webp_bytes'] for r in rows),total_source_bytes=sum(r['source_bytes'] for r in rows))
(ROOT/'manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k!='skins'},ensure_ascii=False))
