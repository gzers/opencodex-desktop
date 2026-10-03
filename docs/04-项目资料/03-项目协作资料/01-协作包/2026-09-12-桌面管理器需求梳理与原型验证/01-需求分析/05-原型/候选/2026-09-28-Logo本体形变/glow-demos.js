/* 背景光 / 动态渐变方案对照：左列表（缩略动效）+ 右详情（模拟概览，单屏）。
   外部方案的观感按官方演示复现，来源在列表与详情上标注。 */
(() => {
  const root = document.documentElement;
  if (root.classList.contains('embedded')) return;

  const REPO = 'https://github.com/';
  const ITEMS = [
    { key:'base', group:'本项目自制 · 对照', name:'上一版 · 无柔化', kind:'hero', glow:'plain',
      src:{ self:true, label:'本项目 IMP-11/12（自制）' },
      short:'容器裁切 + 纵向长渐隐，不做柔化。',
      lead:'光场照原样铺在概览正文列后面，只靠一段纵向长渐隐收尾。',
      approach:'光场容器与概览正文列等宽（889px），能量束和柔光团本身比容器更宽，超出部分由 <code>overflow:hidden</code> 直接裁断；纵向用一段长渐隐（<code>#000 6% → 55% → transparent</code>）把光铺到下方玻璃卡后面，让卡片透光。横向不做任何处理。',
      metric:'边界跳变 13~15/255 · p95 ≈ 26ms',
      cost:'性能最好：没有 filter、没有额外合成层，帧预算最宽松。代价是当能量束转到容器边缘时，边界可能露出一条竖直的裁切直线。',
      recommend:'作为基准线保留。除非能确认实际观感里看不到那条硬边，否则不建议直接当最终方案。' },

    { key:'blur', group:'本项目自制', name:'方案 A · 容器一层模糊', kind:'hero', glow:'blur',
      src:{ self:true, label:'本项目 IMP-12（自制）' },
      short:'整层加一次 filter: blur(20px)。',
      lead:'把模糊加在光场容器这一层，而不是逐个光团。',
      approach:'关键点是位置：<code>blur()</code> 必须加在承载裁切的容器上，容器的裁切刀口才会和光团一起被糊开；如果只模糊每个光团自己，刀口仍然是硬的。纵向仍用长渐隐把光铺到下方卡片。',
      metric:'边界跳变 1~2/255 · p95 ≈ 31ms',
      cost:'边界最柔，几乎看不到直线。代价是整层走离屏合成，每帧多花约 5ms，是三个里最紧的，离 33.4ms 预算只剩 2ms 出头。',
      recommend:'如果确定「边界必须绝对干净」且实机帧预算还有余量，选它；否则优先考虑方案 B。' },

    { key:'gradient', group:'本项目自制', name:'方案 B · 四向渐隐（无模糊）', kind:'hero', glow:'gradient',
      src:{ self:true, label:'本项目 IMP-12（自制）' },
      short:'横向 × 纵向两层遮罩取交集。',
      lead:'用两层遮罩取交集，让光在抵达容器边缘前先淡到 0。',
      approach:'两层 <code>mask-image</code>：一层横向（<code>90deg</code>，两端 transparent），一层纵向（<code>180deg</code>，上下 transparent），用 <code>mask-composite:intersect</code> 取交集。光在碰到容器边界之前就已经淡出，所以从根上不产生刀口，也就不需要 blur。',
      metric:'边界跳变 / p95 待复测',
      cost:'纯渐变、无 filter，理论帧时长应回到现状的 26ms 档；因为横向也渐隐，光场的视觉宽度会比现状略窄一点（边缘更「收」）。',
      recommend:'性能与画质最均衡的一档，当前首选；需要复测确认边界跳变和 p95 后再定稿。' },

    { key:'auroral', group:'外部开源 · 已采用', name:'纯 CSS 极光（auroral）· 正式口径', kind:'hero', glow:'', stars:'287',
      src:{ repo:'LunarLogic/auroral' },
      short:'多层径向渐变 screen 相加 + 容器一次柔化。',
      lead:'已定为正式原型的概览背景光：每个光团由多层径向渐变以 screen 相加成极光带。',
      approach:'采用 LunarLogic/auroral 的做法：每个光团叠两层 <code>radial-gradient</code>，用 <code>background-blend-mode:screen</code> 相加而不是互相覆盖，颜色因此越叠越亮、过渡更连续；柔化只在<b>光场容器</b>上做一次 <code>blur(20px)</code>（不是每层各模糊一次，那样会超帧预算）。颜色仍取状态投影的 <code>--glow-a/b/c</code>，九种状态各自的配色照旧。',
      metric:'纯 CSS · 无 canvas · 容器一次 blur',
      cost:'没有 canvas / WebGL 依赖，低端机与禁用硬件加速时都能降级；代价是一次容器级离屏合成，实测 p95 约 31ms（预算 33.4ms），比无柔化多约 5ms。',
      recommend:'已采用并写入正式原型：<code>原型/index.html</code> 与配对的 <code>候选/…/overview.html</code> 共用同一段 hero 光场，两处同时生效。' },

    { key:'aurora-blue', group:'外部开源 · 已采用', name:'正式口径 · 蓝噪声抖动', kind:'hero', glow:'blue', stars:'—',
      src:{ self:true, label:'自制：void-and-cluster 生成的 64×64 蓝噪声 tile' },
      short:'同 aurora 材质，抖动换成蓝噪声。',
      lead:'材质与 auroral 正式口径完全一致，只把抖动噪声从白噪声换成蓝噪声阈值 tile。',
      approach:'蓝噪声是一张 64×64、256 级的阈值矩阵（用 void-and-cluster 生成，均值 127.5、tile 无缝），以 base64 内联进 CSS，平铺在光场之上、<b>被 blur 的图层之外</b>。它把量化误差推到高频：同样的打散效果下颗粒更细、更不抢眼，所以强度可以比白噪声略高。',
      metric:'纯 CSS · 内联 64×64 蓝噪声 PNG（约 4.2KB）· 无外部请求',
      cost:'比白噪声多 4KB 内联体积；生成脚本一次性、不进入制品。观感上更适合大面积低对比的暗色光场。',
      recommend:'与「正式口径」并排看，选更顺眼的那个；确定后我把选中的抖动方式写进正式口径。' },

    { key:'mesh', group:'外部开源 · 复写观感', name:'Mesh 渐变 + 颗粒着色器', kind:'demo', demo:'mesh', stars:'3.5k',
      src:{ repo:'paper-design/shaders', label:'paper-design/shaders · MeshGradient' },
      short:'三光源公转+自旋；材质＝mesh 渐变 + 颗粒。',
      lead:'同一套「三光源公转 + 自旋」运动，材质换成 mesh 渐变 + 颗粒。',
      approach:'运动沿用概览光场模型（公转 120° 间隔 / 各自自旋 / 按 120° 时差涨缩），在着色器里按同一组参数摆放三个光源。每个光源内部是各向异性椭圆衰减，颜色在光源内做 mesh 式插值；再叠一层哈希颗粒（dither）把 8bit 量化台阶打散。原项目是零依赖的着色器集合（还有 grain / dither）。',
      metric:'WebGL · 片元着色器',
      cost:'画质最干净，颗粒能把暗部色带压住；需要 WebGL 上下文，低端机或软件渲染下要降级。',
      recommend:'如果想把背景光做成「大面积、低对比」的底，这个最耐看；也是四个外部方案里最贴合我们场景的。演示公转半径放大到 210×150，概览实值约 102×70。' },

    { key:'flow', group:'外部开源 · 复写观感', name:'流光网格渐变', kind:'demo', demo:'flow', stars:'2.7k',
      src:{ repo:'ruucm/shadergradient', label:'ruucm/shadergradient' },
      short:'三光源公转+自旋；材质＝噪声揉皱的流光。',
      lead:'同一套运动，材质换成噪声揉皱的流光 + 绕中心自转的能量束。',
      approach:'运动沿用概览光场模型（三光源公转 / 自旋 / 涨缩），另外补上概览里那束绕中心自转的「能量束」。材质用等价片元着色器复现：fbm 噪声揉皱坐标后按正弦场混三到四种颜色，再叠丝绸条纹与暗角。原项目基于 Three.js，未内联。',
      metric:'WebGL · 原版需 Three.js',
      cost:'流动感和色彩浓度最强，适合当「主角」；原版依赖较重（Three.js），纯 CSS 做不到同样的连续形变。',
      recommend:'如果我们想要更有生命力的背景，可以拿它的思路做简化版；直接用原库会明显增加包体。演示公转半径放大到 210×150。' },

    { key:'bits', group:'外部开源 · 复写观感', name:'Aurora / Glow 组件', kind:'demo', demo:'bits', stars:'48k',
      src:{ repo:'DavidHDev/react-bits', label:'DavidHDev/react-bits · Aurora' },
      short:'三光源公转+自旋；材质＝斜向条纹渐变。',
      lead:'同一套「三光源公转 + 自旋」运动，材质换成它公开的斜向条纹渐变。',
      approach:'运动沿用概览光场模型（三光源公转 / 自旋 / 涨缩）。材质按 react-bits Aurora 组件的公开写法复写：底层是斜向 <code>repeating-linear-gradient</code> 彩色条纹，上层叠白色条纹，<code>background-size</code> 拉开后 60s 线性平移，再整体 <code>blur(10px)</code>，深色用 <code>invert()</code>。原项目是 React 组件合集，还含 Grain / Glow 等背景件。',
      metric:'Canvas/CSS · 原版为 React',
      cost:'只用 CSS，兼容性最好、刷新率稳定；观感偏「条纹极光」，没有颗粒，暗部同样可能轻微色带。',
      recommend:'它的 Grain 组件值得单独拿出来做「压色带」的一层，成本比 WebGL 低。演示公转半径放大到 210×150。' }
  ];
  const byKey = Object.fromEntries(ITEMS.map(i => [i.key, i]));
  let selected = 'auroral';
  let dark = document.body.classList.contains('dark');

  const listEl = document.getElementById('glowList');
  const detailEl = document.getElementById('glowDetail');
  const countEl = document.getElementById('glowCount');
  const themeBtn = document.getElementById('glowTheme');
  const pageTheme = document.getElementById('theme');
  const labEl = document.querySelector('.glow-lab');
  if (!listEl || !detailEl) return;

  /* ── 单屏：对照 tab 打开时锁整页滚动，只有左列表滚 ── */
  let detailRO = null;
  function setLock(on) { root.classList.toggle('glow-lock', on); }
  function fitHeight() {
    if (!labEl) return;
    const top = labEl.getBoundingClientRect().top;
    if (top <= 0) return;
    labEl.style.height = Math.max(430, innerHeight - top - 16) + 'px';
  }
  addEventListener('resize', () => { fitHeight(); fitMock(); });
  document.querySelectorAll('[data-lite-tab]').forEach(tab => tab.addEventListener('click', () => {
    const on = tab.dataset.liteTab === 'glow';
    setLock(on);
    requestAnimationFrame(() => { fitHeight(); fitMock(); });
  }));

  /* ── 左列表 ── */
  const groupsSeen = new Set();
  function srcHtml(item) {
    return item.src.self
      ? `<span class="glow-tag">自制</span><span>${item.src.label}</span>`
      : `<span class="glow-tag">复写观感</span><a href="${REPO + item.src.repo}" target="_blank" rel="noreferrer">${item.src.label} ↗</a><span>★ ${item.stars}</span>`;
  }
  ITEMS.forEach(item => {
    if (!groupsSeen.has(item.group)) {
      groupsSeen.add(item.group);
      const h = document.createElement('div'); h.className = 'glow-group'; h.textContent = item.group; listEl.append(h);
    }
    const btn = document.createElement('button');
    btn.type = 'button'; btn.className = 'glow-item'; btn.dataset.key = item.key;
    btn.setAttribute('aria-selected', String(item.key === selected));
    btn.innerHTML = `<span class="glow-thumb" data-thumb="${item.key}"></span>` +
      `<span><b>${item.name}</b><em>${item.short}</em><span class="glow-src">${srcHtml(item)}</span></span>`;
    btn.addEventListener('click', () => { selected = item.key; syncSelection(); renderDetail(); });
    listEl.append(btn);
  });
  function syncSelection() {
    listEl.querySelectorAll('.glow-item').forEach(b => b.setAttribute('aria-selected', String(b.dataset.key === selected)));
    if (countEl) countEl.textContent = `共 ${ITEMS.length} 个方案`;
  }

  /* ── 演示实现：运动全部沿用概览光场模型（morph.js ambientTarget） ──
     三团光源相隔 120° 绕中心 logo 公转、各自自旋、按 120° 时差涨缩；能量束绕中心自转。
     各方案只替换材质/画质。 */
  const ORBIT = { rate:0.5, spin:[42,-33,54], lag:Math.PI*2/3, phase:Math.PI*2/6, energy:0.75 };
  // 演示用公转半径：运动模型与概览同源，半径按演示放大，好让「公转 / 自旋 / 涨缩」看得清
  const ORBIT_RX = 150, ORBIT_RY = 100;   // ≈1.5× 概览实值（102×70），既连续又看得出公转

  function orbitRig(cls) {
    return (host) => {
      const wrap = document.createElement('div'); wrap.className = 'glow-orbit';
      const clouds = [0,1,2].map(i => { const el = document.createElement('i'); el.className = `orbit-cloud ${cls} c${i}`; wrap.append(el); return el; });
      const beam = document.createElement('i'); beam.className = `orbit-beam ${cls}`; wrap.append(beam);
      host.append(wrap);
      let k = 1;
      function setK() { const w = host.clientWidth || 889; k = w / 889; wrap.style.setProperty('--k', String(k)); }
      setK();
      const ro = new ResizeObserver(() => { setK(); }); ro.observe(host);
      let raf = 0; const t0 = performance.now();
      function loop(now) {
        const p = ((now - t0) / 1000) * ORBIT.phase;
        clouds.forEach((el, i) => {
          const orbit = p * ORBIT.rate + i * ORBIT.lag;
          const b = Math.sin(p + i * ORBIT.lag);
          const x = Math.cos(orbit) * ORBIT_RX * k, y = Math.sin(orbit) * ORBIT_RY * k;
          el.style.transform = `translate(${x.toFixed(2)}px,${y.toFixed(2)}px) scale(${(1 + b * 0.22).toFixed(3)}) rotate(${(p * ORBIT.spin[i]).toFixed(1)}deg)`;
          el.style.opacity = String(((dark ? 0.44 : 0.34) + b * 0.07).toFixed(3));
        });
        const bs = 0.9 + ORBIT.energy * 0.2 + Math.sin(p) * 0.08;
        beam.style.transform = `rotate(${(p * 23).toFixed(1)}deg) scale(${bs.toFixed(3)})`;
        beam.style.opacity = String(((0.10 + ORBIT.energy * 0.20) * (1 + Math.sin(p) * 0.14)).toFixed(3));
        raf = requestAnimationFrame(loop);
      }
      raf = requestAnimationFrame(loop);
      return { stop(){ cancelAnimationFrame(raf); ro.disconnect(); }, setTheme(){} };
    };
  }

  const VERT = `attribute vec2 p;void main(){gl_Position=vec4(p,0.,1.);}`;
  const NOISE = `
    float hash(vec2 p){p=fract(p*vec2(127.1,311.7));p+=dot(p,p+34.56);return fract(p.x*p.y);}
    float vnoise(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.0-2.0*f);
      float a=hash(i),b=hash(i+vec2(1.,0.)),c=hash(i+vec2(0.,1.)),d=hash(i+vec2(1.,1.));
      return mix(mix(a,b,f.x),mix(c,d,f.x),f.y);}
    float fbm(vec2 p){float s=0.0,a=0.5;for(int i=0;i<4;i++){s+=a*vnoise(p);p*=2.02;a*=0.5;}return s;}`;
  // 与概览同源的公转/自旋/时差参数（写死在 shader 里，保证与 hero 同一套运动）
  const ORBIT_GLSL = `
    const float LX = 2.0943951;
    vec3 light(vec2 p, float phase, float lag, float spinDeg, vec3 cA, vec3 cB, float warp){
      float orbit = phase*0.5 + lag;
      vec2 o = vec2(cos(orbit)*150.0, sin(orbit)*100.0);
      float rot = phase*spinDeg*0.0174533;
      float sc  = 1.0 + sin(phase+lag)*0.22;
      vec2 d = (p - o) / sc;
      float ca=cos(-rot), sa=sin(-rot);
      vec2 q = vec2(d.x*ca - d.y*sa, d.x*sa + d.y*ca);
      if(warp>0.5) q += (vec2(fbm(q*0.010+vec2(phase*0.3,0.0)), fbm(q*0.010+vec2(5.2,-phase*0.3)))-0.5)*120.0;
      q.x /= 340.0; q.y /= 250.0;
      float w = exp(-dot(q,q)*2.2);
      return mix(cA, cB, clamp(0.5 + 0.5*q.y*1.6, 0.0, 1.0)) * w;
    }
    vec3 beam(vec2 p, float phase, vec3 c, float power){
      float ang = atan(p.y, p.x);
      float g = pow(0.5 + 0.5*sin(ang*3.0 - phase*2.4), 6.0);
      return c * g * power * exp(-length(p)/520.0);
    }
    float fade(float y){ return smoothstep(0.0,0.06,y) * (1.0 - smoothstep(0.55,1.0,y)); }`;

  const MAIN_HEAD = `
    /* gl_FragCoord.y 自下而上；换成自上而下的 uv，光场中心才对得上 Logo，纵向渐隐方向也与 CSS 一致。 */
    vec2 uv = vec2(gl_FragCoord.x/u_res.x, 1.0 - gl_FragCoord.y/u_res.y);
    float px = u_res.x/889.0;
    vec2 p = (uv - vec2(0.5,173.0/660.0)) * u_res.xy / px;
    float phase = u_time * (6.2831853/6.0);`;
  // 背景透明：alpha ＝ 光的覆盖度，底色不参与；这样光效能叠在任意底色（含暗色窗口）上。
  const MAIN_TAIL = `
    col *= fade(uv.y);
    col += (hash(gl_FragCoord.xy)-0.5)*GRAIN;
    col = clamp(col, 0.0, 1.0);
    float a = clamp(max(max(col.r,col.g),col.b) * ALPHA_GAIN, 0.0, 1.0);
    gl_FragColor = vec4(col, a);`;

  const FRAG_MESH_D = `precision highp float;uniform vec2 u_res;uniform float u_time;${NOISE}${ORBIT_GLSL}
    void main(){${MAIN_HEAD}
      vec3 col = vec3(0.0);
      col += light(p,phase,0.0, 42.0, vec3(0.10,0.12,0.32), vec3(0.62,0.24,0.82), 0.0);
      col += light(p,phase,LX,-33.0, vec3(0.62,0.24,0.82), vec3(0.96,0.38,0.46), 0.0);
      col += light(p,phase,LX*2.0,54.0, vec3(0.18,0.72,0.86), vec3(0.28,0.40,0.95), 0.0);
      float GRAIN=0.045; float ALPHA_GAIN=1.35;${MAIN_TAIL} }`;
  const FRAG_MESH_L = `precision highp float;uniform vec2 u_res;uniform float u_time;${NOISE}${ORBIT_GLSL}
    void main(){${MAIN_HEAD}
      vec3 col = vec3(0.0);
      col += light(p,phase,0.0, 42.0, vec3(0.72,0.66,1.00), vec3(0.90,0.72,1.00), 0.0);
      col += light(p,phase,LX,-33.0, vec3(0.90,0.72,1.00), vec3(1.00,0.76,0.80), 0.0);
      col += light(p,phase,LX*2.0,54.0, vec3(0.66,0.90,1.00), vec3(0.74,0.80,1.00), 0.0);
      float GRAIN=0.028; float ALPHA_GAIN=1.10;${MAIN_TAIL} }`;
  const FRAG_FLOW_D = `precision highp float;uniform vec2 u_res;uniform float u_time;${NOISE}${ORBIT_GLSL}
    void main(){${MAIN_HEAD}
      vec3 col = vec3(0.0);
      col += light(p,phase,0.0, 42.0, vec3(0.34,0.16,0.86), vec3(0.98,0.30,0.62), 1.0);
      col += light(p,phase,LX,-33.0, vec3(0.98,0.30,0.62), vec3(1.00,0.66,0.30), 1.0);
      col += light(p,phase,LX*2.0,54.0, vec3(0.10,0.60,0.92), vec3(0.36,0.20,0.88), 1.0);
      col += beam(p, phase, vec3(0.75,0.85,1.0), 0.55);
      float silk = 0.035*sin(length(p)*0.05 - phase*1.4);
      col += silk;
      float GRAIN=0.030; float ALPHA_GAIN=1.35;${MAIN_TAIL} }`;
  const FRAG_FLOW_L = `precision highp float;uniform vec2 u_res;uniform float u_time;${NOISE}${ORBIT_GLSL}
    void main(){${MAIN_HEAD}
      vec3 col = vec3(0.0);
      col += light(p,phase,0.0, 42.0, vec3(0.78,0.70,1.00), vec3(1.00,0.78,0.86), 1.0);
      col += light(p,phase,LX,-33.0, vec3(1.00,0.78,0.86), vec3(1.00,0.88,0.72), 1.0);
      col += light(p,phase,LX*2.0,54.0, vec3(0.78,0.92,1.00), vec3(0.82,0.76,1.00), 1.0);
      col += beam(p, phase, vec3(0.6,0.68,1.0), 0.18);
      float GRAIN=0.024; float ALPHA_GAIN=1.10;${MAIN_TAIL} }`;

  function webglDemo(fragFor) {
    return (host, opts) => {
      const canvas = document.createElement('canvas'); host.append(canvas);
      const gl = canvas.getContext('webgl', { antialias:false, alpha:true, premultipliedAlpha:false, preserveDrawingBuffer:false });
      if (!gl) { return { stop(){}, setTheme(){} }; }
      const sh = (t, src) => { const s = gl.createShader(t); gl.shaderSource(s, src); gl.compileShader(s); return s; };
      let loc;
      function build(d) {
        const prog = gl.createProgram();
        gl.attachShader(prog, sh(gl.VERTEX_SHADER, VERT));
        gl.attachShader(prog, sh(gl.FRAGMENT_SHADER, fragFor(d)));
        gl.linkProgram(prog); gl.useProgram(prog);
        const buf = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, buf);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 3,-1, -1,3]), gl.STATIC_DRAW);
        const p = gl.getAttribLocation(prog, 'p'); gl.enableVertexAttribArray(p); gl.vertexAttribPointer(p, 2, gl.FLOAT, false, 0, 0);
        loc = { res: gl.getUniformLocation(prog,'u_res'), time: gl.getUniformLocation(prog,'u_time') };
      }
      build(opts.dark);
      let raf = 0, t0 = performance.now();
      function resize() { const r = host.getBoundingClientRect(); const dpr = Math.min(1.6, devicePixelRatio || 1);
        canvas.width = Math.max(1, Math.round(r.width*dpr)); canvas.height = Math.max(1, Math.round(r.height*dpr));
        gl.viewport(0,0,canvas.width,canvas.height); }
      function loop(now) { gl.uniform2f(loc.res, canvas.width, canvas.height); gl.uniform1f(loc.time, (now-t0)/1000); gl.drawArrays(gl.TRIANGLES,0,3); raf = requestAnimationFrame(loop); }
      resize(); raf = requestAnimationFrame(loop);
      const ro = new ResizeObserver(resize); ro.observe(host);
      return { stop(){ cancelAnimationFrame(raf); ro.disconnect(); const l = gl.getExtension('WEBGL_lose_context'); l && l.loseContext(); },
        setTheme(d){ build(d); t0 = performance.now(); } };
    };
  }
  const DEMOS = {
    aurora: orbitRig('aurora'),
    bits: orbitRig('bits'),
    mesh: webglDemo(d => d ? FRAG_MESH_D : FRAG_MESH_L),
    flow: webglDemo(d => d ? FRAG_FLOW_D : FRAG_FLOW_L)
  };

  /* ── 缩略 + 详情 ── */
  const mounted = new Map();
  let detailRO2 = null;
  const PRES = { thumb:{ size:44, padding:18 }, detail:{ size:140, padding:103 } };
  const heroUrl = (glow, noLight) => {
    const u = new URL(location.href);
    u.search = '?embed=1&layout=hero&v=dual&effects=high' + (glow ? `&glow=${glow}` : '') + (noLight ? '&light=0' : '');
    return u.href;
  };
  function broadcast(iframe, pres) {
    try { iframe.contentWindow.postMessage({ type:'opencodex-motion-preview', state:'running',
      theme: dark?'dark':'light', active:true, presentation: pres || PRES.detail },
      location.protocol === 'file:' ? '*' : location.origin); } catch (e) {}
  }
  function heroFrame(host, glow, noLight, pres) {
    const iframe = document.createElement('iframe');
    iframe.title = '运行状态形象'; iframe.loading = 'lazy'; iframe.src = heroUrl(glow, noLight);
    iframe.addEventListener('load', () => broadcast(iframe, pres));
    host.append(iframe);
    return iframe;
  }
  function mountEffect(host, item, opts) {
    const pres = (opts && opts.thumb) ? PRES.thumb : PRES.detail;
    if (item.kind === 'hero') {
      const iframe = heroFrame(host, item.glow, false, pres);
      return { stop(){}, push(){ broadcast(iframe, pres); } };
    }
    const demo = DEMOS[item.demo](host, { dark });
    const iframe = heroFrame(host, null, true, pres);
    return { stop(){ demo.stop && demo.stop(); }, push(){ broadcast(iframe, pres); }, setTheme(d){ demo.setTheme && demo.setTheme(d); } };
  }
  ITEMS.forEach(item => {
    const host = listEl.querySelector(`[data-thumb="${item.key}"]`);
    if (host) mounted.set('thumb:' + item.key, mountEffect(host, item, { thumb:true }));
  });

  function detailHtml(item) {
    const modCard = (title, pill, rows, btns) =>
      `<article class="ov-card"><div class="ov-card-head"><h3>${title}</h3><span class="ov-pill">${pill}</span><span class="ov-dots">•••</span></div>` +
      `<div class="ov-rows">${rows.map(r => `<div class="ov-row"><span>${r[0]}</span><b class="${r[2] || ''}">${r[1]}</b></div>`).join('')}</div>` +
      `<div class="ov-btns">${btns.map((btn, i) => `<span class="${i === 0 && btn[1] ? 'primary' : ''}">${btn[0]}</span>`).join('')}</div></article>`;
    return `<div class="detail-head"><b>${item.name}</b><span class="glow-src">${srcHtml(item)}</span><span class="detail-fps" data-fps>帧率采样中…</span></div>` +
      `<div class="detail-stage"><div class="ov-fit"><div class="ov-mock" data-mock>` +
        `<div class="ov-glow" data-glow></div>` +
        `<header class="ov-topbar">` +
          `<div><h1>概览</h1><p>运行状态与常用操作；环境明细收进运行详情。</p></div>` +
          `<div class="ov-wactions"><span class="ov-seg"><i></i><i></i><i class="on"></i></span>` +
            `<span class="ov-icons"><i></i><i class="bell"></i></span></div>` +
        `</header>` +
        `<div class="ov-body">` +
          `<div class="ov-hero"><span class="ov-mark"></span>` +
            `<div class="ov-state">未运行<span>›</span></div>` +
            `<span class="ov-cta">启动 OpenCodex</span></div>` +
          `<div class="ov-mods">` +
            modCard('版本升级', '可用', [['升级状态', '可升级', 'warn'], ['最新版本', '2.51.0']], [['检查更新', 1], ['升级前备份'], ['升级设置']]) +
            modCard('配置迁移', '规划中', [['加密容器', '规划中'], ['导入校验', '规划中']], [['导出配置'], ['导入配置'], ['迁移设置']]) +
            modCard('WebDAV 同步', '未配置', [['连接状态', '未配置'], ['冲突策略', '未设置']], [['配置 WebDAV', 1]]) +
          `</div>` +
          `<div class="ov-events"><div class="ov-events-head"><h3>最近事件</h3><span>打开诊断中心</span></div>` +
            `<div class="ov-event"><time>10:23:41</time><span>config/provider: 已脱敏同步</span></div>` +
            `<div class="ov-event"><time>10:23:40</time><span>health: ready</span></div>` +
          `</div>` +
        `</div>` +
      `</div></div></div>` +
      `<div class="detail-info">` +
        `<div><h4>做法</h4><p>${item.lead}</p><h4 style="margin-top:10px">实现思路</h4><p>${item.approach}</p></div>` +
        `<div><h4>代价与指标</h4><p><b>${item.metric}</b></p><p>${item.cost}</p></div>` +
        `<div><h4>适用建议</h4><p>${item.recommend}</p></div>` +
      `</div>`;
  }
  let fpsWin = null, fpsRaf = 0, fpsTimes = [], fpsEl = null;
  function stopFps() {
    if (fpsWin && fpsRaf) { try { fpsWin.cancelAnimationFrame(fpsRaf); } catch (e) {} }
    fpsRaf = 0; fpsWin = null; fpsTimes = [];
  }
  function startFps(win) {
    stopFps();
    fpsEl = detailEl.querySelector('[data-fps]');
    if (!win) { if (fpsEl) fpsEl.textContent = '帧率不可用'; return; }
    fpsWin = win;
    const tick = now => {
      if (win !== fpsWin) return;
      fpsTimes.push(now);
      while (fpsTimes.length > 2 && now - fpsTimes[0] > 1000) fpsTimes.shift();
      if (fpsTimes.length > 3 && fpsEl) {
        const span = fpsTimes[fpsTimes.length - 1] - fpsTimes[0];
        const fps = span > 0 ? ((fpsTimes.length - 1) * 1000 / span) : 0;
        const d = []; for (let i = 1; i < fpsTimes.length; i++) d.push(fpsTimes[i] - fpsTimes[i - 1]);
        d.sort((a, b) => a - b);
        const p95 = d[Math.min(d.length - 1, Math.floor(d.length * 0.95))] || 0;
        fpsEl.textContent = `帧率 ${Math.round(fps)} FPS · p95 帧间隔 ${p95.toFixed(1)}ms`;
      }
      fpsRaf = win.requestAnimationFrame(tick);
    };
    fpsRaf = win.requestAnimationFrame(tick);
  }

  function bindFps(frameEl) {
    if (!frameEl) return startFps(window);
    const go = () => { try { startFps(frameEl.contentWindow); } catch (e) { startFps(window); } };
    try {
      const w = frameEl.contentWindow;
      if (w && w.location.href !== 'about:blank' && frameEl.contentDocument && frameEl.contentDocument.readyState === 'complete') return go();
    } catch (e) {}
    frameEl.addEventListener('load', go, { once:true });
  }
  function renderDetail() {
    const item = byKey[selected];
    ['detail:'].forEach(p => {});
    [...mounted.keys()].filter(k => k.startsWith('detail:')).forEach(k => { mounted.get(k).stop && mounted.get(k).stop(); mounted.delete(k); });
    detailEl.innerHTML = detailHtml(item);
    const glowHost = detailEl.querySelector('[data-glow]');
    mounted.set('detail:' + item.key, mountEffect(glowHost, item, { thumb:false }));
    // hero 方案量它自己 iframe 里的帧率；外部方案在本页渲染，量本页。
    if (item.kind === 'hero') bindFps(detailEl.querySelector('.ov-glow iframe'));
    else startFps(window);
    applyMockTheme(dark);
    fitMock(); requestAnimationFrame(fitMock); setTimeout(fitMock, 80); setTimeout(fitMock, 400);
    if (!detailRO2 && 'ResizeObserver' in window) {
      const st = detailEl.querySelector('.detail-stage');
      if (st) { detailRO2 = new ResizeObserver(() => fitMock()); detailRO2.observe(st); }
    }
  }
  function fitMock() {
    const stage = detailEl.querySelector('.detail-stage');
    const mock = detailEl.querySelector('[data-mock]');
    const fit = detailEl.querySelector('.ov-fit');
    if (!stage || !mock || !fit) return;
    const s = Math.max(0.1, Math.min(stage.clientWidth / 920, stage.clientHeight / 704) * 0.98);
    mock.style.transform = `scale(${s})`;
    fit.style.width = (920 * s) + 'px'; fit.style.height = (704 * s) + 'px';
  }

  /* ── 主题联动 ── */
  // 主题的真源是「页面实际渲染成什么样」，不是我们自己记的开关：
  // 页面暗色由 body.dark 驱动，任何同步疏漏都会让模拟概览和页面不一致（曾导致暗色下仍是白底）。
  function pageIsDark() {
    try {
      const bg = getComputedStyle(document.body).backgroundColor || '';
      const nums = bg.match(/[\d.]+/g);
      if (!nums || nums.length < 3) return dark;
      const alpha = nums.length > 3 ? parseFloat(nums[3]) : 1;
      if (alpha === 0) return document.body.classList.contains('dark') || root.getAttribute('data-theme') === 'dark';
      const lum = 0.2126 * +nums[0] + 0.7152 * +nums[1] + 0.0722 * +nums[2];
      return lum < 128;
    } catch (e) { return dark; }
  }
  function applyMockTheme(isDark) {
    document.querySelectorAll('.ov-mock').forEach(el => {
      el.classList.toggle('is-dark', isDark);
      el.classList.toggle('is-light', !isDark);
    });
  }
  function syncButtons() {
    if (themeBtn) { themeBtn.setAttribute('aria-pressed', String(dark)); themeBtn.textContent = dark ? '主题：深色' : '主题：浅色'; }
    if (pageTheme) { pageTheme.setAttribute('aria-pressed', String(dark)); pageTheme.textContent = dark ? '浅色' : '深色'; }
  }
  function refreshTheme() {
    dark = pageIsDark();
    applyMockTheme(dark);
    syncButtons();
    mounted.forEach(m => { m.push ? m.push() : (m.setTheme && m.setTheme(dark)); });
  }
  function setDark(v) {
    document.body.classList.toggle('dark', v);
    root.setAttribute('data-theme', v ? 'dark' : 'light');
    refreshTheme();
  }
  if (themeBtn) themeBtn.addEventListener('click', () => setDark(!pageIsDark()));
  if (pageTheme) pageTheme.addEventListener('click', () => setTimeout(refreshTheme, 0));
  root.setAttribute('data-theme', dark ? 'dark' : 'light');
  if ('MutationObserver' in window) {
    const mo = new MutationObserver(() => refreshTheme());
    mo.observe(document.body, { attributes: true, attributeFilter: ['class'] });
    mo.observe(root, { attributes: true, attributeFilter: ['data-theme'] });
  }

  syncSelection(); refreshTheme(); renderDetail(); fitHeight();
})();
