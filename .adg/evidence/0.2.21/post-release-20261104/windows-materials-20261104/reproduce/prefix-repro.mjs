import { createRequire } from 'node:module'
const require=createRequire(new URL('../../post021-windows-materials/apps/desktop/ui/package.json',import.meta.url))
const {transform}=require('lightningcss')
for(const [name,css] of Object.entries({
  standardThenPrefix:'.surface{backdrop-filter:var(--blur);-webkit-backdrop-filter:var(--blur)}',
  prefixThenStandard:'.surface{-webkit-backdrop-filter:var(--blur);backdrop-filter:var(--blur)}',
  differentFallback:'.surface{-webkit-backdrop-filter:var(--blur,none);backdrop-filter:var(--blur)}',
  supportsSeparated:'@supports(-webkit-backdrop-filter:blur(1px)){.surface{-webkit-backdrop-filter:var(--blur)}}@supports(backdrop-filter:blur(1px)){.surface{backdrop-filter:var(--blur)}}',
}))console.log(JSON.stringify({name,input:css,output:transform({filename:'repro.css',code:Buffer.from(css),minify:true}).code.toString()}))
