import { createApp } from 'vue'
import { createPinia } from 'pinia'
import '../src/styles/tokens.css'
import '../src/styles/base.css'
import MeshCenterFixture from './MeshCenter0110Fixture.vue'
// QA-only observation of real GL calls. No renderer/shader replacement.
const names = new WeakMap()
const metrics = { cy: null, draws: 0 }
const proto = WebGLRenderingContext.prototype
const location = proto.getUniformLocation
proto.getUniformLocation = function(program, name) {
  const result = location.call(this, program, name)
  if (result) names.set(result, name)
  return result
}
const uniform = proto.uniform1f
proto.uniform1f = function(location, value) {
  if (location && names.get(location) === 'u_cy') metrics.cy = value
  return uniform.call(this, location, value)
}
const draw = proto.drawArrays
proto.drawArrays = function(...args) { metrics.draws++; return draw.apply(this, args) }
createApp(MeshCenterFixture, { metrics }).use(createPinia()).mount('#app')
