/* Mesh 渐变 + 颗粒着色器（WEBGL 背景光）
   与「纯 CSS 极光（auroral）」并列的另一种背景光渲染方式，默认优先使用。
   运动模型与 hero 光场同源：三团相隔 120° 绕中心 Logo 公转、各自自旋、按 120° 时差涨缩。
   颜色取状态投影 --glow-a/b/c（由调用方传入）。背景透明，只输出光的覆盖度，
   可叠在任意底色（含暗色窗口）上；碎片着色器自带 hash 颗粒抖动，抑制 8bit 色带。
   不支持 WEBGL 时 supported() 返回 false，调用方回退到 CSS 方案。 */
(function(){
  const VERT = `attribute vec2 p;void main(){gl_Position=vec4(p,0.,1.);}`;
  const NOISE = `
    float hash(vec2 p){p=fract(p*vec2(127.1,311.7));p+=dot(p,p+34.56);return fract(p.x*p.y);}
    float vnoise(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.0-2.0*f);
      float a=hash(i),b=hash(i+vec2(1.,0.)),c=hash(i+vec2(0.,1.)),d=hash(i+vec2(1.,1.));
      return mix(mix(a,b,f.x),mix(c,d,f.x),f.y);}
    float fbm(vec2 p){float s=0.0,a=0.5;for(int i=0;i<4;i++){s+=a*vnoise(p);p*=2.02;a*=0.5;}return s;}`;
  const ORBIT_GLSL = `
    const float LX = 2.0943951;
    /* 每团光自带一种状态投影配色（不再是相邻两色的混合——那会让三团颜色趋同、看着像一颗球）。
       团内再叠一道明度梯度（q.y），让单团内部也有层次。 */
    vec3 light(vec2 p, float phase, float lag, float spinDeg, vec3 c){
      float orbit = phase*0.5 + lag;
      vec2 o = vec2(cos(orbit)*195.0, sin(orbit)*132.0);
      float rot = phase*spinDeg*0.0174533;
      float sc  = 1.0 + sin(phase+lag)*0.22;
      vec2 d = (p - o) / sc;
      float ca=cos(-rot), sa=sin(-rot);
      vec2 q = vec2(d.x*ca - d.y*sa, d.x*sa + d.y*ca);
      q.x /= 300.0; q.y /= 250.0;
      float w = exp(-dot(q,q)*1.9);
      return c * w * (0.82 + 0.42 * clamp(0.5 + 0.5*q.y*1.6, 0.0, 1.0));
    }
    /* 顶部渐隐要足够长（原先只有画布高的 5%≈33px，会在顶部切出一条可见的弧线边界）；
       底部同理。两端都用长缓坡，光场就不会出现可辨认的硬边。 */
    float fade(float y){ return sqrt(smoothstep(0.0,0.24,y)) * (1.0 - smoothstep(0.70,1.10,y)); }`;
  const FRAG = `precision highp float;uniform vec2 u_res;uniform float u_time;
    uniform vec3 u_ca;uniform vec3 u_cb;uniform vec3 u_cc;uniform float u_grain;uniform float u_gain;uniform float u_sat;uniform float u_cy;
    ${NOISE}${ORBIT_GLSL}
    void main(){
      /* gl_FragCoord.y 自下而上；统一换成「自上而下」的 uv，中心才落在 Logo 上，纵向渐隐方向也才与 CSS 一致。 */
      vec2 uv = vec2(gl_FragCoord.x/u_res.x, 1.0 - gl_FragCoord.y/u_res.y);
      float px = u_res.x/889.0;
      /* 中心取 logo 实际位置的比例：就绪/准备的舞台高度与 logo 上边距不同，写死 173/660 会把光场整体压低。 */
      vec2 p = (uv - vec2(0.5,u_cy)) * u_res.xy / px;
      float phase = u_time * (6.2831853/6.0);
      vec3 l1 = light(p,phase,0.0, 42.0, u_ca) * 0.62;
      vec3 l2 = light(p,phase,LX,-33.0, u_cb) * 0.62;
      vec3 l3 = light(p,phase,LX*2.0,54.0, u_cc) * 0.62;
      /* 仍是线性加法：screen 在中等重叠处天生比加法暗（两团各 0.4：加法 0.8、screen 只有 0.64），
         整片光会跟着变弱。去圈靠的是下面的软拐点，不是换叠加方式。 */
      vec3 col = l1 + l2 + l3;
      col *= fade(uv.y) * u_gain;
      /* 重叠处趋近白；按亮度做一次饱和度回补，避免整片发灰。 */
      float lum = dot(col, vec3(0.2126,0.7152,0.0722));
      col = mix(vec3(lum), col, u_sat);
      /* 保色相的**带拐点**软压缩：knee 以下原样保留，只在接近饱和处平滑收。
         不能用无拐点的 exp(-peak)：它从 0 就开始压（峰值 0.5 会被压掉约两成），整片光会整体变暗。 */
      float peak = max(max(col.r,col.g),col.b);
      float knee = 0.72;
      if (peak > knee) col *= (knee + (1.0 - knee) * (1.0 - exp(-(peak - knee) / (1.0 - knee)))) / peak;
      col += (hash(gl_FragCoord.xy)-0.5)*u_grain;
      col = clamp(col, 0.0, 1.0);
      float a = clamp(max(max(col.r,col.g),col.b) * 1.3, 0.0, 1.0);
      gl_FragColor = vec4(col, a);
    }`;
  let cached = null;
  function supported(){
    if (cached !== null) return cached;
    try { const c = document.createElement('canvas'); cached = !!(c.getContext('webgl') || c.getContext('experimental-webgl')); }
    catch (e) { cached = false; }
    return cached;
  }
  function rgb(str){
    const m = String(str || '').match(/[\d.]+/g);
    if (!m || m.length < 3) return [0.5, 0.5, 0.5];
    return [ +m[0]/255, +m[1]/255, +m[2]/255 ];
  }
  function mount(host, opts){
    const o = opts || {};
    if (!host || !supported()) return { ok:false, stop(){}, setTheme(){} };
    const canvas = document.createElement('canvas'); canvas.className = 'mesh-glow-canvas';
    host.append(canvas);
    const gl = canvas.getContext('webgl', { antialias:false, alpha:true, premultipliedAlpha:false, preserveDrawingBuffer:false });
    if (!gl) { canvas.remove(); cached = false; return { ok:false, stop(){}, setTheme(){} }; }
    const sh = (t, src) => { const s = gl.createShader(t); gl.shaderSource(s, src); gl.compileShader(s); return s; };
    const prog = gl.createProgram();
    gl.attachShader(prog, sh(gl.VERTEX_SHADER, VERT));
    gl.attachShader(prog, sh(gl.FRAGMENT_SHADER, FRAG));
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) { canvas.remove(); return { ok:false, stop(){}, setTheme(){} }; }
    gl.useProgram(prog);
    const buf = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 3,-1, -1,3]), gl.STATIC_DRAW);
    const ap = gl.getAttribLocation(prog, 'p'); gl.enableVertexAttribArray(ap); gl.vertexAttribPointer(ap, 2, gl.FLOAT, false, 0, 0);
    const loc = { res: gl.getUniformLocation(prog,'u_res'), time: gl.getUniformLocation(prog,'u_time'),
      ca: gl.getUniformLocation(prog,'u_ca'), cb: gl.getUniformLocation(prog,'u_cb'), cc: gl.getUniformLocation(prog,'u_cc'),
      grain: gl.getUniformLocation(prog,'u_grain'), gain: gl.getUniformLocation(prog,'u_gain'), sat: gl.getUniformLocation(prog,'u_sat'),
      cy: gl.getUniformLocation(prog,'u_cy') };
    let dark = !!o.dark, colors = ['#B1A7FF','#7A9DFF','#3941FF'], reduced = false;
    function applyLook(){
      const c = colors.map(rgb);
      gl.uniform3fv(loc.ca, c[0]); gl.uniform3fv(loc.cb, c[1]); gl.uniform3fv(loc.cc, c[2]);
      gl.uniform1f(loc.grain, dark ? 0.045 : 0.028);
      gl.uniform1f(loc.gain, dark ? 0.82 : 0.92);
      gl.uniform1f(loc.sat, dark ? 1.28 : 1.18);
    }
    applyLook();
    function resize(){
      const r = host.getBoundingClientRect();
      const dpr = Math.min(1.6, window.devicePixelRatio || 1);
      canvas.width = Math.max(1, Math.round(r.width*dpr)); canvas.height = Math.max(1, Math.round(r.height*dpr));
      gl.viewport(0,0,canvas.width,canvas.height);
      gl.uniform1f(loc.cy, centerFraction(r));
    }
    /* logo 中心在容器内的纵向比例；量不到就回落到设计稿基准（889×660 的 y=173）。 */
    function centerFraction(rect){
      const fallback = 173/660;
      if (rect.height <= 0) return fallback;
      const svg = host.querySelector('svg');
      if (!svg) return fallback;
      const box = svg.getBoundingClientRect();
      if (box.height <= 0) return fallback;
      return (box.top + box.height/2 - rect.top) / rect.height;
    }
    let raf = 0, t0 = performance.now(), frozen = 0;
    function loop(now){
      gl.uniform2f(loc.res, canvas.width, canvas.height);
      gl.uniform1f(loc.time, reduced ? frozen : (now-t0)/1000);
      gl.drawArrays(gl.TRIANGLES,0,3);
      raf = requestAnimationFrame(loop);
    }
    resize(); raf = requestAnimationFrame(loop);
    const ro = ('ResizeObserver' in window) ? new ResizeObserver(resize) : null;
    if (ro) ro.observe(host);
    return {
      ok:true,
      setTheme(d){ dark = !!d; applyLook(); },
      setColors(list){ if (Array.isArray(list) && list.length >= 3) { colors = list.slice(0,3); applyLook(); } },
      setReduced(v){ if (v) frozen = (performance.now()-t0)/1000; reduced = !!v; },
      pause(){ if (raf) { cancelAnimationFrame(raf); raf = 0; } },
      resume(){ if (!raf) raf = requestAnimationFrame(loop); },
      stop(){ if (raf) cancelAnimationFrame(raf); raf = 0; if (ro) ro.disconnect();
        const l = gl.getExtension('WEBGL_lose_context'); if (l) l.loseContext(); canvas.remove(); }
    };
  }
  window.OpenCodexMeshGlow = { supported, mount };
})();
