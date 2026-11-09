
          const {mkdirSync}=await import('node:fs');
          const {join}=await import('node:path');
          const {pathToFileURL}=await import('node:url');
          const runtimePackage=process.env.OCXD_RECORD_PROBE_PACKAGE;
          const metadata=await Bun.file(join(runtimePackage,'package.json')).json();
          if(metadata.version!=='2.97.0-desktop023.names.2')throw Error('Pinned package required');
          const {atomicWriteFile}=await import(pathToFileURL(join(runtimePackage,'src/config/atomic-write.ts')).href);
          const h=process.env.OPENCODEX_HOME;
          mkdirSync(h,{recursive:true,mode:0o700});
          atomicWriteFile(join(h,'runtime-port.json'),JSON.stringify({pid:process.pid,port:1,attestationSecret:'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'}));
        