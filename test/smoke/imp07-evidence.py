import sys, os, subprocess, re, time, statistics
sys.path.insert(0,'test/smoke/lib')
import ocxui
from PIL import Image
ROOT=os.getcwd(); H=os.path.join(ROOT,'test/runtime/home-imp07'); os.makedirs(H,exist_ok=True)
APP=os.path.join(ROOT,'test/software/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
EV=os.path.join(ROOT,'.adg/work/imp07-execution/evidence'); os.makedirs(EV,exist_ok=True)
p=subprocess.Popen([APP],env=dict(os.environ,HOME=H,OPENCODEX_HOME=H),stdout=open(os.path.join(H,'app.log'),'w'),stderr=subprocess.STDOUT)
def raise_app(): subprocess.run(['osascript','-e','tell application "System Events" to set frontmost of (first process whose name is "%s") to true'%ocxui.APP_NAME],capture_output=True)
def win():
    for line in subprocess.run(['test/smoke/bin/wins','OpenCodeX'],capture_output=True,text=True).stdout.splitlines():
        if 'onscreen=true' in line:
            n=dict(re.findall(r'"(X|Y|Width|Height)":\s*(-?\d+)',line)); return int(n['X']),int(n['Y']),int(n['Width']),int(n['Height'])
def cap(path):
    for _ in range(15):
        raise_app(); time.sleep(0.6); w=win()
        if not w or w[2]<1000: time.sleep(0.5); continue
        x,y,W,Hh=w; subprocess.run(['screencapture','-x','-R','%d,%d,%d,%d'%(x,y,W,Hh),path],capture_output=True)
        if Image.open(path).size==(2*W,2*Hh) and statistics.pstdev(list(Image.open(path).convert('L').getdata()))>12: return w
try:
    for _ in range(40):
        if win(): break
        time.sleep(0.5)
    time.sleep(3)
    nums=win(); ox,oy=nums[0],nums[1]
    cap(os.path.join(EV,'imp07-overview-topbar.png')); print('overview captured')
    subprocess.run(['cliclick','c:%d,%d'%(ox+64,oy+169)]); time.sleep(2.5)  # 侧栏「面板」
    subprocess.run(['cliclick','m:%d,%d'%(ox+1000,oy+700)])                  # 光标移开，避免 tooltip
    time.sleep(1.0)
    cap(os.path.join(EV,'imp07-panel-no-topbar.png')); print('panel captured')
    print('panel OCR:', [r[4] for r in ocxui.ocr(os.path.join(EV,'imp07-panel-no-topbar.png'))][:6])
finally:
    p.terminate()
    try: p.wait(timeout=5)
    except Exception: p.kill()
print('DONE')
