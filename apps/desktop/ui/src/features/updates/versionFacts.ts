import {invoke} from '@tauri-apps/api/core'
export interface RuntimeVersionFacts{
 disk:{diskVersion:string|null;selectedVersion:string|null;recordedVersion:string|null;recordedAt:string|null;integrity:'consistent'|'changed'|'missing'|'pending'|'unavailable';checkedAt:string;confirmationDigest:string}
 running:{version:string;pid:number;port:number}|null
 runningState:'attested'|'no_record'|'unavailable'
}
export function runtimeVersionFacts(){return invoke<RuntimeVersionFacts>('runtime_version_facts').then(value=>{
 if(!value||!value.disk||!['consistent','changed','missing','pending','unavailable'].includes(value.disk.integrity)||!['attested','no_record','unavailable'].includes(value.runningState)||!value.disk.confirmationDigest||value.runningState==='attested'&&(!value.running||!value.running.version||!Number.isSafeInteger(value.running.pid)||value.running.pid<1||!Number.isInteger(value.running.port)||value.running.port<1||value.running.port>65535)||value.runningState!=='attested'&&value.running!==null)throw Error('版本事实尚未核验')
 return value
})}
