import assert from 'node:assert/strict'
import { readFile, writeFile } from 'node:fs/promises'
import { createHash } from 'node:crypto'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
const root = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const source = await readFile(join(code, 'apps/desktop/ui/src/features/runtime/motion/meshGlow.ts'), 'utf8')
const vertex = source.match(/const VERT = '([^']+)'/)[1]
const noise = source.match(/const NOISE = `([\s\S]*?)`/)[1]
const orbit = source.match(/const ORBIT_GLSL = `([\s\S]*?)`/)[1]
const fragment = source.match(/const FRAG = `([\s\S]*?)`/)[1]
  .replace('${NOISE}', noise).replace('${ORBIT_GLSL}', orbit)
const launch = JSON.parse((await readFile(join(root, '../OCX-0110-20261015-WIN-REAL-STATE-FLOOR/state-floor-retry-launch.json'), 'utf8')).replace(/^\uFEFF/, ''))
const targets = await (await fetch(`http://127.0.0.1:${launch.port}/json/list`)).json()
const ws = new WebSocket(targets.find(target => target.type === 'page').webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(Error('WebSocket deadline')), 5000)
  ws.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
})
const expression = `(async () => {
  const vertex=${JSON.stringify(vertex)},fragment=${JSON.stringify(fragment)};
  function setup(depth,width,height){
    const canvas=document.createElement('canvas');canvas.width=width;canvas.height=height;
    const gl=canvas.getContext('webgl',{antialias:false,alpha:true,premultipliedAlpha:false,preserveDrawingBuffer:false,depth,stencil:false});
    if(!gl)throw Error('Real WebGL context unavailable');
    function compile(type,source){const s=gl.createShader(type);gl.shaderSource(s,source);gl.compileShader(s);
      if(!gl.getShaderParameter(s,gl.COMPILE_STATUS))throw Error(gl.getShaderInfoLog(s));return s;}
    const p=gl.createProgram(),v=compile(gl.VERTEX_SHADER,vertex),f=compile(gl.FRAGMENT_SHADER,fragment);
    gl.attachShader(p,v);gl.attachShader(p,f);gl.linkProgram(p);
    if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(p));
    gl.useProgram(p);gl.deleteShader(v);gl.deleteShader(f);
    const b=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,b);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,3,-1,-1,3]),gl.STATIC_DRAW);
    const position=gl.getAttribLocation(p,'p');gl.enableVertexAttribArray(position);gl.vertexAttribPointer(position,2,gl.FLOAT,false,0,0);
    const loc=Object.fromEntries(['res','time','ca','cb','cc','grain','gain','sat','cy'].map(k=>[k,gl.getUniformLocation(p,'u_'+k)]));
    gl.uniform2f(loc.res,width,height);gl.viewport(0,0,width,height);
    gl.uniform3fv(loc.ca,[177/255,167/255,1]);gl.uniform3fv(loc.cb,[122/255,157/255,1]);gl.uniform3fv(loc.cc,[57/255,65/255,1]);
    return{canvas,gl,p,b,loc,attributes:gl.getContextAttributes(),depthBits:gl.getParameter(gl.DEPTH_BITS)};
  }
  const cases=[];
  for(const [width,height]of[[128,96],[320,240]]){
    const left=setup(true,width,height),right=setup(false,width,height);
    try{
      for(const dark of[false,true])for(const time of[0,0.5,3])for(const cy of[173/660,0.4]){
        const pixels=[];
        for(const x of[left,right]){
          const g=x.gl;g.uniform1f(x.loc.time,time);g.uniform1f(x.loc.cy,cy);
          g.uniform1f(x.loc.grain,dark?0.045:0.028);g.uniform1f(x.loc.gain,dark?0.82:0.92);g.uniform1f(x.loc.sat,dark?1.28:1.18);
          g.drawArrays(g.TRIANGLES,0,3);g.finish();
          const a=new Uint8Array(width*height*4);g.readPixels(0,0,width,height,g.RGBA,g.UNSIGNED_BYTE,a);
          if(g.getError()!==g.NO_ERROR)throw Error('WebGL error while comparing actual pixels');pixels.push(a);
        }
        let differentBytes=0,maxDifference=0,nonzeroBytes=0;
        for(let i=0;i<pixels[0].length;i++){const d=Math.abs(pixels[0][i]-pixels[1][i]);if(d)differentBytes++;maxDifference=Math.max(maxDifference,d);if(pixels[0][i])nonzeroBytes++;}
        const hashes=[];
        for(const bytes of pixels){const hash=await crypto.subtle.digest('SHA-256',bytes);hashes.push([...new Uint8Array(hash)].map(x=>x.toString(16).padStart(2,'0')).join(''));}
        cases.push({width,height,dark,time,centerFraction:cy,leftDepthBits:left.depthBits,rightDepthBits:right.depthBits,
          leftDepthAttribute:left.attributes.depth,rightDepthAttribute:right.attributes.depth,differentBytes,maxDifference,nonzeroBytes,rgbaSha256:hashes});
      }
    }finally{for(const x of[left,right]){x.gl.deleteBuffer(x.b);x.gl.deleteProgram(x.p);x.gl.getExtension('WEBGL_lose_context')?.loseContext();x.canvas.remove();}}
  }
  return{atUtc:new Date().toISOString(),caseCount:cases.length,cases,liveProductCanvasTouched:false,
    attachedToDocument:false,systemTimerChanged:false,productionCodeChanged:false};
})()`
ws.send(JSON.stringify({ id: 1, method: 'Runtime.evaluate', params: { expression, awaitPromise: true, returnByValue: true } }))
try {
  const value = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(Error('Pixel comparison deadline')), 30000)
    ws.addEventListener('message', event => {
      const response = JSON.parse(event.data)
      if (response.id !== 1) return
      clearTimeout(timer); resolve(response)
    })
  })
  await writeFile(join(root, 'real-webgl-depth-pixels.original.json'), JSON.stringify({
    sourceSha256: createHash('sha256').update(source).digest('hex'),
    vertexSha256: createHash('sha256').update(vertex).digest('hex'),
    fragmentSha256: createHash('sha256').update(fragment).digest('hex'),
    originalProtocolResult: value,
  }, null, 2) + '\n', { flag: 'wx' })
  assert(!value.error && !value.result.exceptionDetails)
  const result = value.result.result.value
  assert.equal(result.caseCount, 24)
  for (const test of result.cases) {
    assert(test.leftDepthAttribute && !test.rightDepthAttribute)
    assert(test.leftDepthBits > 0 && test.rightDepthBits === 0)
    assert(test.nonzeroBytes > 0)
    assert.equal(test.differentBytes, 0)
    assert.equal(test.rgbaSha256[0], test.rgbaSha256[1])
  }
  console.log('24 real WebGL shader cases: identical RGBA with depth buffer disabled; original product canvas untouched.')
} finally { ws.close() }
