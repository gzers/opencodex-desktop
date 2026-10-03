import sys, os, subprocess, re, time, statistics
sys.path.insert(0,'test/smoke/lib')
import ocxui
from PIL import Image
from collections import Counter
ROOT=os.getcwd(); H=os.path.join(ROOT,'test/runtime/home-imp06')
APP=os.path.join(ROOT,'test/software/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
env=dict(os.environ, HOME=H, OPENCODEX_HOME=H)
p=subprocess.Popen([APP],env=env,stdout=open(os.path.join(H,'app.log'),'w'),stderr=subprocess.STDOUT)
def raise_app(): subprocess.run(['osascript','-e','tell application "System Events" to set frontmost of (first process whose name is "%s") to true'%ocxui.APP_NAME],capture_output=True)
def win():
    for line in subprocess.run(['test/smoke/bin/wins','OpenCodeX'],capture_output=True,text=True).stdout.splitlines():
        if 'onscreen=true' in line:
            n=dict(re.findall(r'"(X|Y|Width|Height)":\s*(-?\d+)',line)); return int(n['X']),int(n['Y']),int(n['Width']),int(n['Height'])
def capture(path):
    for _ in range(12):
        raise_app(); time.sleep(0.6); w=win()
        if not w or w[2]<1000: time.sleep(0.5); continue
        x,y,W,Hh=w
        subprocess.run(['screencapture','-x','-R','%d,%d,%d,%d'%(x,y,W,Hh),path],capture_output=True)
        if Image.open(path).size==(2*W,2*Hh) and statistics.pstdev(list(Image.open(path).convert('L').getdata()))>12: return w
def locate_refresh(path):
    im=Image.open(path).convert('RGB'); reg=im.crop((0,90,im.size[0],200)); px=list(reg.getdata())
    bg=Counter(px).most_common(1)[0][0]; W,Hh=reg.size
    ink=[sum(1 for y in range(Hh) if sum(abs(reg.getpixel((x,y))[i]-bg[i]) for i in range(3))>60) for x in range(W)]
    segs=[];s=None
    for x,v in enumerate(ink):
        if v>1 and s is None:s=x
        elif v<=1 and s is not None:segs.append((s,x));s=None
    if s is not None:segs.append((s,W))
    segs=[(a,b) for a,b in segs if b-a>=4 and a/2>900]
    # 取中间那个 32 宽左右的
    for a,b in segs:
        if 26<=(b-a)/2<=40: return (a+b)/4.0  # css center x
    return None
try:
    for _ in range(40):
        if win(): break
        time.sleep(0.5)
    time.sleep(3)
    hit=ocxui.click_text('设置','test/out/_nav-settings.png'); print('click 设置 ->',hit)
    time.sleep(2.5)
    capture('test/out/imp06-04-settings.png')
    print('settings OCR', [r[4] for r in ocxui.ocr('test/out/imp06-04-settings.png')][:5])
    cx=locate_refresh('test/out/imp06-04-settings.png'); print('refresh css cx',cx)
    if cx:
        x,y,W,Hh=win(); tx,ty=int(x+cx),int(y+73)
        subprocess.run(['cliclick','m:%d,%d'%(tx,ty)]); time.sleep(0.3)
        idle='test/out/imp06-04-settings-idle.png'
        subprocess.run(['screencapture','-x','-R','%d,%d,50,50'%(x+int(cx)-25,y+48),idle],capture_output=True)
        base=list(Image.open(idle).convert('L').getdata())
        print('busy inkΔ vs idle:')
        for i in range(24):
            f='test/out/anim/s%02d.png'%i
            if i==5: subprocess.run(['cliclick','c:%d,%d'%(tx,ty)])
            subprocess.run(['screencapture','-x','-R','%d,%d,50,50'%(x+int(cx)-25,y+48),f],capture_output=True)
            cur=list(Image.open(f).convert('L').getdata())
            print('  s%02d %4d'%(i,sum(1 for a,b in zip(base,cur) if abs(a-b)>30)))
        raise_app(); time.sleep(0.2)
        subprocess.run(['screencapture','-x','-R','%d,%d,%d,%d'%(x,y,W,Hh),'test/out/imp06-04-settings-refreshing.png'],capture_output=True)
finally:
    p.terminate()
    try: p.wait(timeout=5)
    except Exception: p.kill()
print('DONE')
