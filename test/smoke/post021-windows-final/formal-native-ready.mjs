import assert from 'node:assert/strict'
export async function waitForNativeReady(evaluate,{now=Date.now,sleep=ms=>new Promise(r=>setTimeout(r,ms)),timeoutMs=30000}={}){
 const deadline=now()+timeoutMs
 while(now()<deadline){
  if(await evaluate(()=>!!window.__TAURI_INTERNALS__?.invoke && !!document.querySelector('#app')?.__vue_app__?.config?.globalProperties?.$pinia))return
  await sleep(200)
 }
 assert.fail('Native Tauri/Vue bootstrap not ready; no native command performed')
}
