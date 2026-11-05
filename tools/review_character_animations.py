"""Export a reproducible, game-scale cast comparison against the merged baseline."""
import argparse
import io
import json
import subprocess
from pathlib import Path

from prepare_character_animations import Image, ROOT, ROLES, source_dir
from PIL import ImageDraw


def frame(atlas, index, columns):
    x, y = index % columns * 384, index // columns * 384
    return atlas.crop((x, y, x + 384, y + 384)).resize((160, 160), Image.Resampling.NEAREST)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True, help="External folder with retained baseline atlases; discarded artwork is not in Git")
    args = parser.parse_args()
    baseline = args.baseline.resolve()
    out = ROOT / ".local/analysis/issue11-animation"
    out.mkdir(parents=True, exist_ok=True)
    images = []
    for role in ROLES:
        old = Image.open(baseline / f"{role}.webp").convert("RGBA")
        new = Image.open(ROOT / f"assets/roles/{role}/runtime/atlas.webp").convert("RGBA")
        images.append((role, old, new))
    for awake in (False, True):
        sequence = []
        for step in range(12):
            canvas = Image.new("RGB", (430, 900), "#20232d")
            draw = ImageDraw.Draw(canvas)
            draw.text((95, 12), "BEFORE: 4 POSES", fill="#e0b876")
            draw.text((270, 12), "AFTER: 12 POSES", fill="#8be3cf")
            for row, (role, old, new) in enumerate(images):
                draw.text((12, 45 + row * 170), role, fill="white")
                old_start = 12 if awake and role != "fire-elf" else 8
                new_start = 36 if awake and role != "fire-elf" else 24
                for x, image in ((80, frame(old, old_start + step // 3, 4)),
                                 (255, frame(new, new_start + step, 6))):
                    canvas.paste(image, (x, 35 + row * 170), image)
            sequence.append(canvas)
        name = "awake-cast" if awake else "normal-cast"
        sequence[0].save(out / f"{name}-comparison.webp", "WEBP", save_all=True,
                         append_images=sequence[1:], duration=[117] * 8 + [116] * 3 + [916],
                         loop=0, lossless=True, exact=True, method=6)
    # Single contact sheet is useful in clients that do not animate WebP.
    contact = Image.new("RGB", (12 * 130, 5 * 280), "#20232d")
    draw = ImageDraw.Draw(contact)
    for row, (role, _, atlas) in enumerate(images):
        draw.text((0, row * 280), role + " / cast + awakened cast (fire-elf: sleep)", fill="white")
        for clip in range(2):
            for col in range(12):
                image = frame(atlas, 24 + clip * 12 + col, 6).resize((128, 128), Image.Resampling.NEAREST)
                contact.paste(image, (col * 130, row * 280 + clip * 130 + 20), image)
    contact.save(out / "cast-contact.webp", "WEBP", lossless=True, method=6)
    meta = {role: json.loads((source_dir(role) / "report.json").read_text()) for role in ROLES}
    page = r'''<!doctype html><meta charset="utf-8"><title>Issue 11 · 动作评审</title>
<style>body{background:#20232d;color:#eee;font:16px system-ui;margin:32px}button,select,input{font:inherit;margin:8px}canvas{display:block;max-width:100%;image-rendering:pixelated}small{color:#b5b9c5}</style>
<h1>新五人 · 12 帧动作</h1><p>待机循环；施法／入睡播放一次后停留。拖动时间条可逐帧检查。</p>
<select id="clip"><option value="normal-idle">普通待机</option><option value="awake-idle">觉醒待机</option><option value="normal-cast" selected>普通施法</option><option value="awake-cast">觉醒施法（火法师：入睡）</option></select>
<button id="replay">重播</button><button id="pause">暂停</button><input id="seek" type="range" min="0" max="2000" value="0"><output id="time"></output>
<canvas id="view" width="1000" height="240"></canvas><small>384px 原帧缩至 192px；同一运行时图集，最近邻采样。完整来源及检查记录见角色 production 目录。</small>
<script>const metadata=METADATA;const roles=Object.keys(metadata);const images={};const view=document.querySelector('#view'),ctx=view.getContext('2d');let start=performance.now(),elapsed=0,playing=true;
Promise.all(roles.map(role=>new Promise(resolve=>{const img=new Image();img.onload=()=>{images[role]=img;resolve()};img.src='../../../assets/roles/'+role+'/runtime/atlas.webp'}))).then(()=>requestAnimationFrame(draw));
document.querySelector('#replay').onclick=()=>{elapsed=0;start=performance.now();playing=true};document.querySelector('#pause').onclick=()=>{playing=!playing;start=performance.now()-elapsed};
document.querySelector('#clip').onchange=()=>{elapsed=0;start=performance.now()};document.querySelector('#seek').oninput=e=>{elapsed=+e.target.value;playing=false};
function draw(now){if(playing)elapsed=now-start;ctx.fillStyle='#20232d';ctx.fillRect(0,0,1000,240);ctx.imageSmoothingEnabled=false;roles.forEach((role,r)=>{let name=document.querySelector('#clip').value;if(role==='fire-elf'&&name==='awake-cast')name='sleep';const clip=metadata[role].clips[name];let pos=clip.playback==='loop'?(elapsed%clip.duration_ms):Math.min(elapsed,clip.duration_ms-1);let index=clip.indices[Math.min(11,Math.floor(pos/clip.duration_ms*12))];ctx.drawImage(images[role],index%6*384,Math.floor(index/6)*384,384,384,r*200,30,192,192);ctx.fillStyle='white';ctx.font='14px system-ui';ctx.fillText(role+' · '+(clip.indices.indexOf(index)+1)+'/12',r*200+10,20)});document.querySelector('#seek').value=Math.min(2000,elapsed);document.querySelector('#time').textContent=(elapsed/1000).toFixed(2)+'s';requestAnimationFrame(draw)}
</script>'''
    (out / "index.html").write_text(page.replace("METADATA", json.dumps(meta)), encoding="utf-8")
    print(f"Review exports: {out}; baseline {baseline}")


if __name__ == "__main__":
    main()
