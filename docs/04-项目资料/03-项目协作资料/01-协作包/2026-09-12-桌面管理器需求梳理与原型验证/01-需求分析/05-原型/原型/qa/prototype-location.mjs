import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
let root=path.dirname(fileURLToPath(import.meta.url));
while(!fs.existsSync(path.join(root,'.git'))){const up=path.dirname(root);if(up===root)throw Error('仓库根不存在');root=up;}
export const repoRoot=root;
export const prototypeRoot=path.join(root,'docs/04-项目资料/01-项目支撑资料/01-产品原型');
const retiredRoot=path.join(root,'docs/04-项目资料/03-项目协作资料/03-共享原型');
if(fs.existsSync(retiredRoot))throw Error('产品原型目录必须唯一：请清理旧共享原型目录');
export const prototypeEntry=path.join(prototypeRoot,'原型/index.html');
if(!fs.existsSync(prototypeEntry))throw Error('产品原型主入口不存在');
// 兼容既有 QA 的导出名；实际位置统一到产品原型。
export const sharedRoot=prototypeRoot;
export const sharedPrototype=prototypeEntry;
