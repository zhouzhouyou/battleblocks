"""Bake layered pixel healing overlays. PixelFX holy/heal adaptation; see licenses/pixelfx/."""
from pathlib import Path
import math,json
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'assets-source/effects/nun-heal'
PREVIEWS=ROOT/'.local/previews/nun_heal'
PREVIEWS.mkdir(parents=True,exist_ok=True)
W,H,COUNT,COLS=160,192,40,8
TAU=math.tau

def frame(i,hurt,front):
    im=Image.new('RGBA',(W,H));d=ImageDraw.Draw(im)
    t=i/20
    if i==COUNT-1:return im
    pal=['#30203f','#643351','#a34573','#cc698c','#e9a5c8','#fff0fa'] if hurt else ['#593442','#a56747','#d99c58','#f5ca7d','#fff0cf','#fffaf2']
    fade=min(1,t/.15,max(0,(1.95-t)/.5))
    def color(k,a=1):return tuple(bytes.fromhex(pal[k][1:]))+(round(255*fade*a),)
    def line(points,k,width=1,a=1):d.line([(round(x),round(y)) for x,y in points],fill=color(k,a),width=width)
    # Eight light petals bloom around the feet; front and back follow their depth.
    for k in range(8):
        g=k*TAU/8+.2+t*.22
        if (math.sin(g)>=0)!=front:continue
        b=min(1,max(0,(t-k%3*.035)/.32))*min(1,max(0,(1.8-t)/.55))
        r=38*b;cx=80;cy=176
        tip=(cx+math.cos(g)*r,cy+math.sin(g)*r*.26-18*b)
        base=(cx+math.cos(g)*11,cy+math.sin(g)*3)
        dx,dy=tip[0]-base[0],tip[1]-base[1];length=max(1,math.hypot(dx,dy));nx,ny=-dy/length*7*b,dx/length*7*b
        mid=((tip[0]+base[0])/2,(tip[1]+base[1])/2)
        pts=[base,(mid[0]+nx,mid[1]+ny),tip,(mid[0]-nx,mid[1]-ny)]
        d.polygon([(round(x),round(y)) for x,y in pts],fill=color(1,.65),outline=color(3))
        line([base,tip],4)
        # Short stepped inner vein adds a material ramp without soft glow.
        line([(mid[0]+nx*.3,mid[1]+ny*.3),tip],5,1,.8)
    # Body-height hoop travels upward when restoring and downward when corrupted.
    if .12<=t<=1.45:
        p=min(1,(t-.12)/1.1);p=(1-math.cos(p*math.pi))/2
        y=176-128*(1-p if hurt else p);radius=38-7*p
        for j in range(90):
            g=j*TAU/90;near=math.sin(g)>=0
            if near!=front:continue
            if hurt and j%23<4:continue
            x=80+math.cos(g)*radius;yy=y+math.sin(g)*9
            line([(x,yy+2),(x+1,yy+2)],2,2)
            line([(x,yy),(x+1,yy)],5 if near else 3,2)
    # Ordered spiral motes, then small hearts / split hearts. No random pixel noise.
    for k in range(12):
        age=t-(.16+k*.065)
        if not 0<age<.95:continue
        p=age/.95;g=k*TAU/8+age*3.1
        if (math.sin(g)>=0)!=front:continue
        x=round(80+math.cos(g)*(30-8*p));y=round(170-139*(1-p if hurt else p))
        a=min(1,age/.12,(.95-age)/.22)
        line([(x,y+5 if not hurt else y-5),(x,y)],2,2,a)
        if k%3==0 and p>.4:
            heart=['.#.#.','#####','.###.','..#..']
            for yy,row in enumerate(heart):
                for xx,v in enumerate(row):
                    if v=='#':
                        shift=(2 if xx>=2 else -1) if hurt else 0
                        d.rectangle((x+(xx-2)*2+shift,y+(yy-2)*2,x+(xx-2)*2+shift+1,y+(yy-2)*2+1),fill=color(4 if yy else 5,a))
        else:
            line([(x-2,y),(x+2,y)],4,1,a);line([(x,y-2),(x,y+2)],5,1,a)
    return im

role=Image.open(ROOT/'assets/roles/nun/runtime/atlas.webp').convert('RGBA')
all_previews=[]
for hurt,name in [(False,'normal'),(True,'damage')]:
    layers=[[frame(i,hurt,front) for i in range(COUNT)] for front in [False,True]]
    atlas=Image.new('RGBA',(W*COLS,H*5*2))
    for layer,frames in enumerate(layers):
        for i,im in enumerate(frames):atlas.paste(im,((i%COLS)*W,(i//COLS+layer*5)*H))
    target=ROOT/f'assets/effects/nun-heal-{name}.webp'
    atlas.save(target,lossless=True,exact=True,method=6)
    assert Image.open(target).convert('RGBA').tobytes()==atlas.tobytes()
    previews=[]
    for i in range(COUNT):
        im=Image.new('RGBA',(224,224),'#252633')
        im.alpha_composite(layers[0][i],(32,16))
        index=(8 if hurt else 0)+i//5%4
        sprite=role.crop(((index%4)*384,(index//4)*384,(index%4+1)*384,(index//4+1)*384)).resize((213,213),Image.Resampling.NEAREST)
        im.alpha_composite(sprite,(6,208-186));im.alpha_composite(layers[1][i],(32,16))
        previews.append(im.convert('RGB'))
    previews[0].save(PREVIEWS/f'{name}-preview.webp',save_all=True,append_images=previews[1:],duration=50,loop=0,lossless=True,method=6)
    all_previews.append(previews)
contact=Image.new('RGB',(224*6,248*2),'#161820');d=ImageDraw.Draw(contact)
for row,previews in enumerate(all_previews):
    for col,i in enumerate([3,8,14,20,28,35]):
        contact.paste(previews[i],(col*224,row*248+24));d.text((col*224+8,row*248+6),f'{["HEAL","CORRUPTED"][row]} {i/20:.2f}s',fill='#fff0cf')
contact.save(PREVIEWS/'contact.webp',lossless=True)
(OUT/'atlas.json').write_text(json.dumps(dict(width=W,height=H,frames=COUNT,columns=COLS,layer_rows=5,layers=['back','front'],fps=20,duration=2,origin=[80,176],reference='pixelfx holy/heal; MIT; see licenses/pixelfx',character_assets_modified=False),indent=2)+'\n')
print('Baked normal / damage lossless atlases and previews')
