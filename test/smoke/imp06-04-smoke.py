import sys, os, subprocess, re, time, statistics
sys.path.insert(0,'test/smoke/lib')
import ocxui
from PIL import Image
from collections import Counter

ROOT=os.getcwd(); H=os.path.join(ROOT,'test/runtime/home-imp06')
APP=os.path.join(ROOT,'test/software/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
env=dict(os.environ, HOME=H, OPENCODEX_HOME=H)
p=subprocess.Popen([APP],env=env,stdout=open(os.path.join(H,'app.log'),'w'),stderr=subprocess.STDOUT)

def raise_app():
    subprocess.run(['osascript','-e','tell application "System Events" to set frontmost of (first process whose name is "%s") to true'%ocxui.APP_NAME],capture_output=True)

def win():
    for line in subprocess.run(['test/smoke/bin/wins','OpenCodeX'],capture_output=True,text=True).stdout.splitlines():
        if 'onscreen=true' in line:
            n=dict(re.findall(r'"(X|Y|Width|Height)":\s*(-?\d+)',line))
            return int(n['X']),int(n['Y']),int(n['Width']),int(n['Height'])
    return None

def capture(path):
    for _ in range(12):
        raise_app(); time.sleep(0.6)
        w=win()
        if not w or w[2]<1000 or w[3]<600: time.sleep(0.6); continue
        x,y,W,Hh=w
        subprocess.run(['screencapture','-x','-R','%d,%d,%d,%d'%(x,y,W,Hh),path],capture_output=True)
        im=Image.open(path)
        if im.size==(2*W,2*Hh):
            st=statistics.pstdev(list(im.convert('L').getdata()))
            if st>12: return w,round(st,2)
    return None,None

def segs(path,Y0,Y1):
    im=Image.open(path).convert('RGB'); reg=im.crop((0,Y0,im.size[0],Y1)); px=list(reg.getdata())
    bg=Counter(px).most_common(1)[0][0]; W,Hh=reg.size
    ink=[sum(1 for y in range(Hh) if sum(abs(reg.getpixel((x,y))[i]-bg[i]) for i in range(3))>60) for x in range(W)]
    out=[];s=None
    for x,v in enumerate(ink):
        if v>1 and s is None:s=x
        elif v<=1 and s is not None:out.append((s,x));s=None
    if s is not None:out.append((s,W))
    return [(round(a/2,1),round(b/2,1)) for a,b in out if b-a>=4]

try:
    for _ in range(40):
        if win(): break
        time.sleep(0.5)
    time.sleep(3)
    w,st=capture('test/out/imp06-04-overview.png')
    print('overview window',w,'std',st)
    print('  OCR', [r[4] for r in ocxui.ocr('test/out/imp06-04-overview.png')][:6])
    print('  topbar-right ink', [s for s in segs('test/out/imp06-04-overview.png',90,200) if s[0]>900])
    hit=ocxui.click_text('面板','test/out/_nav-panel.png'); print('click 面板 ->',hit)
    time.sleep(2.5)
    w,st=capture('test/out/imp06-04-panel.png')
    print('panel window',w,'std',st)
    print('  OCR', [r[4] for r in ocxui.ocr('test/out/imp06-04-panel.png')][:5])
    print('  topbar-right ink', [s for s in segs('test/out/imp06-04-panel.png',90,200) if s[0]>900])
    # 刷新态：区域快速循环
    x,y,W,Hh=w; ox,oy=x,y
    tx,ty=ox+1103,oy+73
    subprocess.run(['cliclick','m:%d,%d'%(tx,ty)]); time.sleep(0.3)
    idle='test/out/imp06-04-panel-idle.png'
    subprocess.run(['screencapture','-x','-R','%d,%d,50,50'%(ox+1080,oy+50),idle],capture_output=True)
    base=list(Image.open(idle).convert('L').getdata())
    print('busy inkΔ:')
    for i in range(26):
        f='test/out/anim/f%02d.png'%i
        if i==5: subprocess.run(['cliclick','c:%d,%d'%(tx,ty)])
        subprocess.run(['screencapture','-x','-R','%d,%d,50,50'%(ox+1080,oy+50),f],capture_output=True)
        cur=list(Image.open(f).convert('L').getdata())
        print('  f%02d %4d'%(i,sum(1 for a,b in zip(base,cur) if abs(a-b)>30)))
    raise_app(); time.sleep(0.3)
    subprocess.run(['screencapture','-x','-R','%d,%d,%d,%d'%(ox,oy,W,Hh),'test/out/imp06-04-panel-refreshing.png'],capture_output=True)
finally:
    p.terminate()
    try: p.wait(timeout=5)
    except Exception: p.kill()
print('DONE')
