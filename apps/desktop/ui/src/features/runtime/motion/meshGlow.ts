// 背景光「网格渐变 + 颗粒着色器」渲染器（WEBGL）。
//
// 与 `2026-09-28-Logo本体形变/mesh-glow.js` 同一套实现与参数（已按原型验收定值）：
// 三团光沿用概览光场同源的运动模型（120° 公转 / 各自自旋 / 时差涨缩），颜色取状态投影
// `--glow-a/b/c`；加法叠加处做保色相归一，避免叠成纯白；碎片着色器自带颗粒抖动抑制色带。
// 只在设备支持 WEBGL 时创建；创建失败由调用方回退纯 CSS 极光。
//
// 说明：这是装饰层实现，不采集状态、不推定操作成功。

const VERT = 'attribute vec2 p;void main(){gl_Position=vec4(p,0.,1.);}'

const NOISE = `
  float hash(vec2 p){p=fract(p*vec2(127.1,311.7));p+=dot(p,p+34.56);return fract(p.x*p.y);}
  float vnoise(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.0-2.0*f);
    float a=hash(i),b=hash(i+vec2(1.,0.)),c=hash(i+vec2(0.,1.)),d=hash(i+vec2(1.,1.));
    return mix(mix(a,b,f.x),mix(c,d,f.x),f.y);}
  float fbm(vec2 p){float s=0.0,a=0.5;for(int i=0;i<4;i++){s+=a*vnoise(p);p*=2.02;a*=0.5;}return s;}`

const ORBIT_GLSL = `
  const float LX = 2.0943951;
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
  /* 顶部渐隐要足够长（过短会在顶部切出一条可见的弧线边界）；两端都用长缓坡。 */
  float fade(float y){ return sqrt(smoothstep(0.0,0.24,y)) * (1.0 - smoothstep(0.70,1.10,y)); }`

const FRAG = `precision highp float;uniform vec2 u_res;uniform float u_time;
  uniform vec3 u_ca;uniform vec3 u_cb;uniform vec3 u_cc;uniform float u_grain;uniform float u_gain;uniform float u_sat;uniform float u_cy;
  ${NOISE}${ORBIT_GLSL}
  void main(){
    /* gl_FragCoord.y 自下而上；换成自上而下的 uv，光场中心才对得上 Logo，纵向渐隐方向也与 CSS 一致。 */
    vec2 uv = vec2(gl_FragCoord.x/u_res.x, 1.0 - gl_FragCoord.y/u_res.y);
    float px = u_res.x/889.0;
    /* 中心取「logo 实际所在位置」的比例，而不是写死设计稿的 173/660：
       就绪/准备、标准/紧凑四种形态的光场高度与 logo 上边距都不同，写死会把光场整体压低。 */
    vec2 p = (uv - vec2(0.5,u_cy)) * u_res.xy / px;
    float phase = u_time * (6.2831853/6.0);
    vec3 l1 = light(p,phase,0.0, 42.0, u_ca) * 0.62;
    vec3 l2 = light(p,phase,LX,-33.0, u_cb) * 0.62;
    vec3 l3 = light(p,phase,LX*2.0,54.0, u_cc) * 0.62;
    /* 仍是线性加法：screen 在中等重叠处天生比加法暗（两团各 0.4：加法 0.8、screen 只有 0.64），
       整片光会跟着变弱。去圈靠的是下面的软拐点，不是换叠加方式。 */
    vec3 col = l1 + l2 + l3;
    col *= fade(uv.y) * u_gain;
    /* 加法叠加在重叠处趋近白；按亮度回补饱和度，避免整片发灰。 */
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
  }`

export interface MeshGlowHandle {
  setColors(colors: readonly string[]): void
  setReduced(reduced: boolean): void
  setActive(active: boolean): void
  destroy(): void
}

export interface MeshGlowOptions {
  /** 亮色/暗色由调用方给出（跟随主题），只影响增益与颗粒强度。 */
  getDark: () => boolean
  /** logo 中心在光场容器内的纵向比例（0–1）；缺省回落到设计稿的 173/660。 */
  getCenterFraction?: () => number
}

function rgbFloats(value: string): [number, number, number] {
  const parts = String(value ?? '').match(/[\d.]+/g)
  if (!parts || parts.length < 3) return [0.5, 0.5, 0.5]
  return [Number(parts[0]) / 255, Number(parts[1]) / 255, Number(parts[2]) / 255]
}

/**
 * 在 `host` 内创建网格渐变着色器层。返回 `null` 表示当前环境无法创建（调用方应回退 CSS）。
 */
export function createMeshGlow(host: HTMLElement, options: MeshGlowOptions): MeshGlowHandle | null {
  const canvas = document.createElement('canvas')
  canvas.className = 'motion-mesh'
  canvas.setAttribute('aria-hidden', 'true')
  host.append(canvas)

  const context = canvas.getContext('webgl', {
    antialias: false,
    alpha: true,
    premultipliedAlpha: false,
    preserveDrawingBuffer: false,
  }) as WebGLRenderingContext | null
  if (!context) {
    canvas.remove()
    return null
  }
  const gl: WebGLRenderingContext = context

  const compile = (type: number, source: string): WebGLShader | null => {
    const shader = gl.createShader(type)
    if (!shader) return null
    gl.shaderSource(shader, source)
    gl.compileShader(shader)
    return gl.getShaderParameter(shader, gl.COMPILE_STATUS) ? shader : null
  }

  const program = gl.createProgram()
  const vertex = compile(gl.VERTEX_SHADER, VERT)
  const fragment = compile(gl.FRAGMENT_SHADER, FRAG)
  if (!program || !vertex || !fragment) {
    canvas.remove()
    return null
  }
  gl.attachShader(program, vertex)
  gl.attachShader(program, fragment)
  gl.linkProgram(program)
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    canvas.remove()
    return null
  }
  gl.useProgram(program)

  const buffer = gl.createBuffer()
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer)
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW)
  const position = gl.getAttribLocation(program, 'p')
  gl.enableVertexAttribArray(position)
  gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0)

  const loc = {
    res: gl.getUniformLocation(program, 'u_res'),
    time: gl.getUniformLocation(program, 'u_time'),
    ca: gl.getUniformLocation(program, 'u_ca'),
    cb: gl.getUniformLocation(program, 'u_cb'),
    cc: gl.getUniformLocation(program, 'u_cc'),
    grain: gl.getUniformLocation(program, 'u_grain'),
    gain: gl.getUniformLocation(program, 'u_gain'),
    sat: gl.getUniformLocation(program, 'u_sat'),
    cy: gl.getUniformLocation(program, 'u_cy'),
  }

  let colors: readonly string[] = ['#B1A7FF', '#7A9DFF', '#3941FF']
  let reduced = false
  let frame = 0
  let start = performance.now()
  let frozen = 0

  function applyLook(): void {
    const dark = options.getDark()
    const [a, b, c] = [rgbFloats(colors[0]), rgbFloats(colors[1]), rgbFloats(colors[2])]
    gl.uniform3fv(loc.ca, a)
    gl.uniform3fv(loc.cb, b)
    gl.uniform3fv(loc.cc, c)
    gl.uniform1f(loc.grain, dark ? 0.045 : 0.028)
    gl.uniform1f(loc.gain, dark ? 0.82 : 0.92)
    gl.uniform1f(loc.sat, dark ? 1.28 : 1.18)
  }

  function resize(): void {
    const rect = host.getBoundingClientRect()
    const dpr = Math.min(1.6, window.devicePixelRatio || 1)
    canvas.width = Math.max(1, Math.round(rect.width * dpr))
    canvas.height = Math.max(1, Math.round(rect.height * dpr))
    gl.viewport(0, 0, canvas.width, canvas.height)
    gl.uniform1f(loc.cy, centerFraction(rect))
  }

  /** logo 中心在容器内的纵向比例；量不到就回落到设计稿基准（889×660 的 y=173）。 */
  function centerFraction(rect: DOMRect): number {
    const fallback = 173 / 660
    if (rect.height <= 0) return fallback
    const custom = options.getCenterFraction?.()
    if (typeof custom === 'number' && custom > 0 && custom < 1) return custom
    const svg = host.parentElement?.querySelector('.motion-hero svg')
    if (!svg) return fallback
    const box = svg.getBoundingClientRect()
    if (box.height <= 0) return fallback
    return (box.top + box.height / 2 - rect.top) / rect.height
  }

  function loop(now: number): void {
    gl.uniform2f(loc.res, canvas.width, canvas.height)
    gl.uniform1f(loc.time, reduced ? frozen : (now - start) / 1000)
    gl.drawArrays(gl.TRIANGLES, 0, 3)
    frame = requestAnimationFrame(loop)
  }

  function stop(): void {
    if (frame) cancelAnimationFrame(frame)
    frame = 0
  }

  applyLook()
  resize()
  frame = requestAnimationFrame(loop)
  const observer = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(() => resize())
  observer?.observe(host)

  return {
    setColors(next) {
      if (!Array.isArray(next) || next.length < 3) return
      colors = next.slice(0, 3)
      applyLook()
    },
    setReduced(value) {
      if (value && !reduced) frozen = (performance.now() - start) / 1000
      reduced = value
    },
    setActive(value) {
      if (value) {
        if (!frame) {
          start = performance.now() - (reduced ? frozen : 0) * 1000
          frame = requestAnimationFrame(loop)
        }
      } else {
        stop()
      }
    },
    destroy() {
      stop()
      observer?.disconnect()
      const lose = gl.getExtension('WEBGL_lose_context')
      lose?.loseContext()
      canvas.remove()
    },
  }
}
