import sys, os, subprocess, re, time, statistics, json
sys.path.insert(0,'test/smoke/lib')
import ocxui
from PIL import Image
from collections import Counter
ROOT=os.getcwd(); H=os.path.join(ROOT,'test/runtime/home-imp07')
os.makedirs(H, exist_ok=True)
APP=os.path.join(ROOT,'test/software/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
env=dict(os.environ, HOME=H, OPENCODEX_HOME=H)
p=subprocess.Popen([APP],env=env,stdout=open(os.path.join(H,'app.log'),'w'),stderr=subprocess.STDOUT)
def raise_app(): subprocess.run(['osascript','-e','tell application "System Events" to set frontmost of (first process whose name is "%s") to true'%ocxui.APP_NAME],capture_output=True)
def win():
    for line in subprocess.run(['test/smoke/bin/wins','OpenCodeX'],capture_output=True,text=True).stdout.splitlines():
        if 'onscreen=true' in line:
            n=dict(re.findall(r'"(X|Y|Width|Height)":\s*(-?\d+)',line)); return int(n['X']),int(n['Y']),int(n['Width']),int(n['Height'])
def capture(path):
    for _ in range(15):
        raise_app(); time.sleep(0.6); w=win()
        if not w or w[2]<1000: time.sleep(0.5); continue
        x,y,W,Hh=w
        subprocess.run(['screencapture','-x','-R','%d,%d,%d,%d'%(x,y,W,Hh),path],capture_output=True)
        im=Image.open(path)
        if im.size==(2*W,2*Hh) and statistics.pstdev(list(im.convert('L').getdata()))>12: return w
    return None
try:
    for _ in range(40):
        if win(): break
        time.sleep(0.5)
    time.sleep(3)
    w=capture('test/out/imp07-overview.png'); print('overview win',w)
    print('  OCR', [r[4] for r in ocxui.ocr('test/out/imp07-overview.png')][:6])
    hit=ocxui.click_text('面板','test/out/_nav-panel.png'); print('click 面板 ->',hit)
    time.sleep(2.5)
    w=capture('test/out/imp07-panel.png'); print('panel win',w)
    rows=ocxui.ocr('test/out/imp07-panel.png')
    print('  OCR (top 8)', [r[4] for r in rows][:8])
    im=Image.open('test/out/imp07-panel.png').convert('RGB')
    # 底部右侧区域找“彩色”像素（品牌渐变 logo），报告簇位置
    W,Hh=im.size
    bx0,by0,bx1,by1=int(W*0.80),int(Hh*0.80),W,Hh
    reg=im.crop((bx0,by0,bx1,by1)); px=list(reg.getdata()); rw,rh=reg.size
    cols=[]
    for x in range(rw):
        c=0
        for y in range(rh):
            r,g,b=reg.getpixel((x,y))
            if max(r,g,b)-min(r,g,b)>40 and (r+g+b)<720: c+=1
        cols.append(c)
    segs=[];s=None
    for x,v in enumerate(cols):
        if v>2 and s is None:s=x
        elif v<=2 and s is not None:segs.append((s,x));s=None
    if s is not None:segs.append((s,rw))
    print('  hub 彩色簇(css, 相对窗口):', [((bx0+a)/2,(bx0+b)/2) for a,b in segs if b-a>=4])
    print('  window size css', W/2, Hh/2)
finally:
    p.terminate()
    try: p.wait(timeout=5)
    except Exception: p.kill()
print('DONE')
