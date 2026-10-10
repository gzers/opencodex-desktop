/* 0.1.10 目录规则候选；只构造路径，不检测文件、权限或迁移数据。 */
(function (scope) {
  function paths(platform, options={}) {
    if (!["macos", "windows"].includes(platform)) throw new Error("unsupported platform");
    const windows=platform==="windows", separator=windows?"\\":"/";
    const trim=value=>String(value).replace(/[\\/]+$/, "");
    const installation=trim(options.installation || (windows?"E:\\Tools\\OpenCodex Desktop":"/Applications/OpenCodex Desktop.app"));
    const defaultRoot=windows?installation+"\\data":"~/Library/Application Support/com.gzers.opencodex.desktop";
    const root=trim(options.root || defaultRoot), join=part=>root+separator+part.split("/").join(separator);
    return Object.freeze({platform,installation,defaultRoot,root,custom:root!==defaultRoot,
      home:options.home || join("opencodex-home"),
      prefix:options.prefix || join("runtime/opencodex"),entry:join(windows?"runtime/bin/ocx.cmd":"runtime/bin/ocx"),
      preferences:join("manager-state/preferences.json"),backups:join("backups"),
      manager:join("manager-state"),logs:join("logs"),exports:join("exports"),cache:join("cache"),sync:join("sync-state"),
      credentials:windows?"Windows 凭据管理器":"macOS 钥匙串"});
  }
  const api={paths};
  if(typeof module!=="undefined" && module.exports) module.exports=api;
  else scope.DataPaths=api;
})(typeof window!=="undefined"?window:globalThis);
