// Observe the public component rather than attempting to overwrite Tauri's
// non-writable invoke property. Completion still needs OS generation evidence.
export function observePublicRestart(){
 const dialog=[...document.querySelectorAll('dialog[open]')].find(d=>d.querySelector('.ui-dialog-header')?.textContent.includes('重启 Codex'))
 const error=dialog?.querySelector('[role="alert"]')?.textContent.trim()
 if(error)return{state:'failed',error}
 const p=document.querySelector('#app')?.__vue_app__?.config?.globalProperties?.$pinia
 const noticeOpen=p?._s.get('model-save-feedback')?.open
 const text=[...document.querySelectorAll('.toast,.ui-toast,[role="status"]')].map(e=>e.textContent).join(' ')
 if(!dialog&&noticeOpen===false&&text.includes('Codex')&&text.includes('重启'))return{state:'ui-confirmed',noticeOpen,toastText:text}
 return{state:'pending',noticeOpen,dialogOpen:!!dialog}
}
