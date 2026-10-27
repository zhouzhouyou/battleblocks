"""Inventory texture bytes and exact-color rectangles; does not modify game assets."""
from pathlib import Path
from PIL import Image
import json
ROOT = Path(__file__).resolve().parents[1]
rows = []
for path in sorted((ROOT/'assets').glob('*.webp')):
    image = Image.open(path).convert('RGBA')
    # Invisible RGB is not part of the rendered appearance.
    image.putdata([p if p[3] else (0,0,0,0) for p in image.getdata()])
    rgba = image.tobytes(); factor = 1
    for scale in range(2,33):
        if image.width % scale or image.height % scale: continue
        reduced = image.resize((image.width//scale,image.height//scale),Image.Resampling.NEAREST)
        if reduced.resize(image.size,Image.Resampling.NEAREST).tobytes() == rgba: factor = scale
    metadata_path = path.with_suffix('.json')
    frame_width = json.loads(metadata_path.read_text())['frame_size'][0] if metadata_path.exists() else image.width
    rectangle_counts = []
    for offset in range(0,image.width,frame_width):
        frame = image.crop((offset,0,offset+frame_width,image.height)); pixels = list(frame.getdata()); active = set(); count = 0
        for y in range(frame.height):
            runs = set(); x = 0
            while x < frame.width:
                color = pixels[y*frame.width+x]; end = x+1
                while end < frame.width and pixels[y*frame.width+end] == color: end += 1
                if color[3]: runs.add((x,end,color))
                x = end
            count += len(runs-active); active = runs
        rectangle_counts.append(count)
    rows.append(dict(file=path.name,width=image.width,height=image.height,encoded_bytes=path.stat().st_size,rgba_bytes=len(rgba),lossless_integer_scale=factor,downscaled_rgba_bytes=len(rgba)//factor**2,exact_rectangles_per_unique_frame=rectangle_counts))
(ROOT/'analysis/asset-inventory.json').write_text(json.dumps(rows,indent=2));print(json.dumps(rows,indent=2))
