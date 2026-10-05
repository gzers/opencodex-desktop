// 历史 QA 保留所属 COL；持续维护的入口及共用资源移到共享区。
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
let root=path.dirname(fileURLToPath(import.meta.url));
while(!fs.existsSync(path.join(root,'.git'))){const up=path.dirname(root);if(up===root)throw Error('找不到仓库根');root=up;}
export const sharedRoot=path.join(root,'docs/04-项目资料/03-项目协作资料/03-共享原型');
export const sharedPrototype=path.join(sharedRoot,'原型/index.html');
