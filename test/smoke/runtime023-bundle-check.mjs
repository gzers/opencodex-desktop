/** Verify the frozen descriptor and optionally execute only isolated read-only CLIs.
 * --root points at the installation root containing runtime/, never a user home.
 * Execution copies the delivered tree to a relocated installation, generates a
 * test-only absolute launcher there, and leaves the original archive tree intact.
 */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import {
  cpSync, existsSync, lstatSync, mkdirSync, readFileSync, readlinkSync,
  realpathSync, readdirSync, writeFileSync, chmodSync,
} from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep, posix } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export async function sha256(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

function safePath(name) {
  assert.equal(typeof name, 'string');
  assert(name.startsWith('runtime/') && !name.includes('\\') && !name.includes(':'));
  assert.equal(posix.normalize(name), name);
  assert(!name.split('/').includes('..') && !isAbsolute(name));
}

function inside(root, path) {
  const name = relative(realpathSync(root), realpathSync(path));
  assert(name && !name.startsWith('..'+sep) && name !== '..' && !isAbsolute(name));
}

export async function verifyTree(root, descriptor) {
  assert.equal(descriptor.schemaVersion, 1);
  assert(['darwin-arm64', 'win32-x64'].includes(descriptor.platform));
  const adopted=JSON.parse(readFileSync(new URL('../../.github/runtime-platforms-023.json',import.meta.url),'utf8'));
  assert.equal(descriptor.runtimeVersion,adopted.runtime.version);
  assert.equal(descriptor.runtimeArchiveSha256,adopted.runtime.sha256);
  assert.equal(descriptor.nodeVersion, '26.8.1');
  assert.equal(descriptor.bunVersion, '1.4.4');
  assert.match(descriptor.archiveSha256, /^[a-f0-9]{64}$/);
  assert.match(descriptor.runtimeArchiveSha256, /^[a-f0-9]{64}$/);
  assert(descriptor.archiveBytes > 0 && descriptor.archiveBytes <= 512*1024*1024);
  assert(descriptor.entries.length > 0 && descriptor.entries.length <= 100000);
  const paths = descriptor.entries.map(row=>row.path);
  assert.deepEqual(paths, [...new Set(paths)].sort());
  let bytes=0;
  for (const row of descriptor.entries) {
    safePath(row.path);
    const path=join(root,row.path), info=lstatSync(path);
    inside(join(root,'runtime'),path);
    if(row.kind==='file') {
      assert(info.isFile() && !info.isSymbolicLink());
      assert.equal(info.nlink,1,'Hardlinks forbidden');
      assert.equal(info.size,row.bytes);
      assert([0o644,0o755].includes(row.mode));
      if(process.platform!=='win32') assert.equal(info.mode&0o777,row.mode);
      assert.equal(await sha256(path),row.sha256,row.path);
      bytes+=row.bytes;
    } else {
      assert.equal(row.kind,'symlink');
      assert(info.isSymbolicLink());
      assert.equal(readlinkSync(path),row.target);
      assert(!isAbsolute(row.target) && !row.target.includes('\\') && !row.target.includes(':'));
      inside(join(root,'runtime'),path);
    }
  }
  assert(bytes<=2*1024*1024*1024);
  const found=[];
  function visit(directory) {
    for(const name of readdirSync(directory)) {
      const path=join(directory,name), info=lstatSync(path);
      if(info.isDirectory()) visit(path);
      else found.push(relative(root,path).split(sep).join('/'));
    }
  }
  visit(join(root,'runtime'));
  assert.deepEqual(found.sort(),paths,'Descriptor must cover the exact complete tree');
  assert(!paths.some(p=>p==='runtime/bin/ocx'||p==='runtime/bin/ocx.cmd'||p.endsWith('/_runtime-manifest.json')));
  const win=descriptor.platform==='win32-x64';
  const node=win?'runtime/node/node.exe':'runtime/node/bin/node';
  const npm=win?'runtime/node/node_modules/npm/bin/npm-cli.js':'runtime/node/lib/node_modules/npm/bin/npm-cli.js';
  const bun=win?'runtime/bun/bun.exe':'runtime/bun/bun';
  for(const name of [node,npm,bun,'runtime/opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs']) {
    assert(paths.includes(name),'Missing deployment entry '+name);
  }
  return {entries:paths.length,bytes,node,npm,bun};
}

function quote(value) {return "'"+value.replaceAll("'","'\\''")+"'";}

export async function executeReadonly(root, descriptor, evidence) {
  assert.equal(descriptor.platform,process.platform+'-'+process.arch,'Foreign native execution forbidden');
  assert(!existsSync(evidence),'Fresh execution evidence directory required');
  mkdirSync(evidence,{recursive:true});
  const relocated=join(evidence,'relocated I with spaces');
  mkdirSync(relocated);
  cpSync(join(root,'runtime'),join(relocated,'runtime'),{recursive:true,dereference:false});
  const win=process.platform==='win32';
  const node=join(relocated,win?'runtime/node/node.exe':'runtime/node/bin/node');
  const bun=join(relocated,win?'runtime/bun/bun.exe':'runtime/bun/bun');
  const npm=join(relocated,win?'runtime/node/node_modules/npm/bin/npm-cli.js':'runtime/node/lib/node_modules/npm/bin/npm-cli.js');
  const program=join(relocated,'runtime/opencodex/node_modules/@bitkyc08/opencodex');
  const script=join(program,'bin/ocx.mjs');
  const H=join(evidence,'H');
  for(const name of ['home','opencodex','codex','claude','empty-path','tmp']) mkdirSync(join(H,name),{recursive:true});
  const config=join(H,'config-to-validate.json');
  writeFileSync(config,JSON.stringify({port:4096,defaultProvider:'offline',providers:{offline:{adapter:'openai-chat',baseUrl:'https://runtime023.invalid/v1',models:['test'],liveModels:false}}}));
  const before=readFileSync(config);
  const env={HOME:join(H,'home'),USERPROFILE:join(H,'home'),OPENCODEX_HOME:join(H,'opencodex'),
    CODEX_HOME:join(H,'codex'),CLAUDE_CONFIG_DIR:join(H,'claude'),PATH:join(H,'empty-path'),
    TMPDIR:join(H,'tmp'),TEMP:join(H,'tmp'),OCX_NO_DESKTOP_HANDOFF:'1',
    OCX_TEST_HOME_GUARD:'1',OCX_REAL_HOME:process.env.OCX_REAL_HOME??process.env.HOME??'',
    npm_config_userconfig:join(H,'empty.npmrc'),npm_config_globalconfig:join(H,'global.npmrc'),
    npm_config_update_notifier:'false',BUN_RUNTIME_TRANSPILER_CACHE_PATH:'0'};
  writeFileSync(env.npm_config_userconfig,'');writeFileSync(env.npm_config_globalconfig,'');
  if(win) env.SystemRoot=process.env.SystemRoot;
  const bin=join(relocated,'runtime/bin');mkdirSync(bin);
  const launcher=join(bin,win?'ocx.cmd':'ocx');
  // No OPENCODEX_BUN_PATH override here: prove the original Node launcher finds
  // the explicitly materialized bun/bin/bun.exe binary without PATH/postinstall.
  writeFileSync(launcher,win
    ? '@echo off\r\nsetlocal DisableDelayedExpansion\r\n"'+node.replaceAll('%','%%')+'" "'+script.replaceAll('%','%%')+'" %*\r\nexit /b %errorlevel%\r\n'
    : '#!/bin/sh\nexec '+quote(node)+' '+quote(script)+' "$@"\n');
  chmodSync(launcher,0o755);
  let sandbox=[];
  if(!win) {
    assert(existsSync('/usr/bin/sandbox-exec'),'Read-only CLI execution requires macOS network sandbox');
    const policy=join(evidence,'readonly.sb');
    // Network deny includes outbound inference and listening. Real configuration
    // trees are unreadable even if a future CLI bypasses HOME selection.
    const realHome=process.env.OCX_REAL_HOME??process.env.HOME;
    const forbidden=realHome?['.opencodex','.codex','.claude'].map(p=>'(deny file-read* (subpath '+JSON.stringify(join(realHome,p))+'))').join('\n'):'';
    writeFileSync(policy,'(version 1)\n(allow default)\n(deny network*)\n'+forbidden+'\n');
    sandbox=['/usr/bin/sandbox-exec','-f',policy];
  }
  const results=[];
  function run(label,command,args,expected) {
    const argv=[...sandbox,command,...args];
    const result=spawnSync(argv[0],argv.slice(1),{cwd:H,env,encoding:'utf8',timeout:45000,maxBuffer:1024*1024,windowsHide:true});
    const output=(result.stdout??'')+(result.stderr??'');
    writeFileSync(join(evidence,label+'.log'),output);
    assert.equal(result.error,undefined,label+': '+result.error);
    assert.equal(result.status,0,label+': '+output);
    if(expected) assert.match(output,expected);
    results.push({label,exitCode:result.status,stdout:result.stdout.trim()});
  }
  run('node-version',node,['--version'],/^v26\.8\.1\s*$/);
  run('npm-version',node,[npm,'--version'],/^11\.19\.0\s*$/);
  run('bun-version',bun,['--version'],/^1\.4\.4\s*$/);
  const nativePackages=[['@napi-rs/keyring',win?'@napi-rs/keyring-win32-x64-msvc/keyring.win32-x64-msvc.node':'@napi-rs/keyring-darwin-arm64/keyring.darwin-arm64.node'],
    ['wreq-js',win?'wreq-js/rust/wreq-js.win32-x64-msvc.node':'wreq-js/rust/wreq-js.darwin-arm64.node']];
  const nativePaths=nativePackages.map(([name,native])=>[name,join(dirname(dirname(program)),native)]);
  const requireCode='const r=require("node:module").createRequire('+JSON.stringify(join(program,'package.json'))+');for(const [n,native] of '+JSON.stringify(nativePaths)+'){const m=r(n);if(!m||Object.keys(m).length===0)throw Error(n+" unavailable");const p=r.resolve(native);if(!p.endsWith(".node"))throw Error("Not native");r(p);console.log(n+" loaded native "+p);}';
  run('node-native-load',node,['-e',requireCode],/@napi-rs\/keyring loaded/);
  run('bun-native-load',bun,['--no-env-file','-e',requireCode],/wreq-js loaded/);
  if(win) {
    const cmd=join(env.SystemRoot,'System32/cmd.exe');
    run('ocx-version',cmd,['/d','/s','/c','""'+launcher+'" --version"'],new RegExp('^opencodex '+descriptor.runtimeVersion.split('.').join('\\.')+'\\s*$'));
    run('ocx-config-validate',cmd,['/d','/s','/c','""'+launcher+'" config validate "'+config+'" --json"'],/"ok"\s*:\s*true/);
  } else {
    run('ocx-version',launcher,['--version'],new RegExp('^opencodex '+descriptor.runtimeVersion.split('.').join('\\.')+'\\s*$'));
    run('ocx-config-validate',launcher,['config','validate',config,'--json'],/"ok"\s*:\s*true/);
  }
  assert.deepEqual(readFileSync(config),before,'Read-only config input changed');
  assert(!existsSync(join(H,'opencodex/proxy.pid')) && !existsSync(join(H,'opencodex/runtime-port.json')));
  const summary={platform:descriptor.platform,execution:'passed-isolated-readonly-cli',network:win?'not-sandboxed':'denied-by-macos-sandbox',
    externalPath:'empty',relocated:true,managedLauncher:'generated-only-in-test-deployment-context',checks:results};
  writeFileSync(join(evidence,'summary.json'),JSON.stringify(summary,null,2)+'\n');
  return summary;
}

async function main() {
  const args=process.argv.slice(2);
  const get=name=>{const i=args.indexOf(name);return i<0?undefined:args[i+1];};
  const root=resolve(get('--root'));
  const descriptor=JSON.parse(readFileSync(get('--descriptor'),'utf8'));
  const checked=await verifyTree(root,descriptor);
  console.log(JSON.stringify({platform:descriptor.platform,structuralChecks:'passed',...checked}));
  if(args.includes('--execute')) console.log(JSON.stringify(await executeReadonly(root,descriptor,resolve(get('--evidence')))));
}
if(process.argv[1] && resolve(process.argv[1])===fileURLToPath(import.meta.url)) await main();
