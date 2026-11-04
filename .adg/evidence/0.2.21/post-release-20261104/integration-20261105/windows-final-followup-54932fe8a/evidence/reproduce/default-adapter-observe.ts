import {writeFileSync} from 'node:fs'
import {windowsDesktopAppAdapter as adapter,windowsDefaultExec as exec} from '../post021-restart-panel/tools/node_modules/@bitkyc08/opencodex/src/codex/desktop-app/windows.ts'
const install=adapter.discover(exec)
const processes=install?adapter.listProcesses(exec,install):null
const result={mode:'read-only unmodified official Windows default adapter discover/list; no restart requested',install,processes,defaultSelectionScope:'current-user package install root; no CODEX_HOME/USERPROFILE admission',relaunchMechanism:'shell:AppsFolder AUMID; no isolated user-data argument',blocked:!!processes?.length,reason:processes?.length?'Existing packaged desktop processes cannot be proved disposable or isolated from daily Codex; do not call restart ladder':'No isolated packaged desktop normal restart proven'}
writeFileSync(import.meta.dir+'/default-adapter-observation.json',JSON.stringify(result,null,2))
console.log(JSON.stringify(result,null,2))
