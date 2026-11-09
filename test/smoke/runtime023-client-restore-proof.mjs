// Fresh synthetic profile only. This proves file effects, not just an API reply.
export function injectedClientRoute(document,port){
 if(!document||typeof document!=='object'||typeof document.openai_base_url!=='string')return false;
 try{const url=new URL(document.openai_base_url);return url.protocol==='http:'&&url.hostname==='127.0.0.1'&&url.port===String(port)&&!url.username&&!url.password;}catch{return false;}
}
export function restoredFreshNativeClient(document){
 return document!==null&&typeof document==='object'&&document.model_provider==='openai'
  &&!Object.hasOwn(document,'openai_base_url')&&!Object.hasOwn(document,'experimental_realtime_ws_base_url')
  &&!Object.hasOwn(document,'model_catalog_json');
}
