const fs=require('node:fs'),path=require('node:path'),read=fs.readFileSync;
const control=read.call(fs,path.join(__dirname,'model-config-without-transport-hooks.js'),'utf8');
fs.readFileSync=function(file,...args){const result=read.call(this,file,...args);return String(file).endsWith('/原型/model-config.js')&&typeof result==='string'?control:result;};
