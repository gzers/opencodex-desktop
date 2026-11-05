import {connect}from '../../../post021-windows-materials/test/smoke/post021-windows-final/cdp-session.mjs'
const c=await connect('http://127.0.0.1:9450');try{await c.evaluate("document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('notifications').panelOpen=true;true")}finally{c.close()}
