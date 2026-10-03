import { createApp } from 'vue'
import GalleryApp from './GalleryApp.vue'
import '@/styles/tokens.css'
import '@/styles/base.css'
import '@/styles/materials.css'
import '@/styles/effects.css'

// 预览页默认高档；与生产一致由档位按钮重写 `data-effects`（生产由外观策略唯一写入）。
document.documentElement.setAttribute('data-effects', 'high')
createApp(GalleryApp).mount('#gallery')
