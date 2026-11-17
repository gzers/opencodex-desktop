from pathlib import Path
import json
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
p=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/capture.rs'
s=p.read_text(encoding='utf-8');assert s.count('    routes: RoutePin,')==1
s=s.replace('    routes: RoutePin,','    routes: Option<RoutePin>,\n    route_close:Option<super::super::v2::routes::RetainedRouteClose>,',1)
assert s.count('                    routes,')==1
s=s.replace('                    routes,','                    routes:Some(routes),\n                    route_close:None,',1)
s=s.replace('        self.routes\n            .declared','        self.routes.as_mut().ok_or_else(||refused("原路由pin已转移或缺席"))?\n            .declared',1)
s=s.replace('&mut store.routes','store.routes.as_mut().ok_or_else(||refused("原路由pin已转移或缺席"))?')
s=s.replace('                .routes\n                .declared','                .routes.as_mut().ok_or_else(||refused("原路由pin已转移或缺席"))?\n                .declared')
marker='            let mut guard = super::journal::CompletedStopGuard::bind('
assert s.count(marker)==1
pre='''            if store.route_close.is_some() {return Err(refused("原路由关闭对象已有状态"));}
            // Move exact owned RoutePin into a retained slot BEFORE prepare or
            // any fallible route-close IO. No Drop release or replacement.
            let routes=store.routes.take().ok_or_else(||refused("原路由pin缺席"))?;
            store.route_close=Some(super::super::v2::routes::RetainedRouteClose::retain(routes,end));
            store.route_close.as_mut().ok_or_else(||refused("原路由关闭槽缺席"))?.prepare(end).map_err(refused)?;
'''
s=s.replace(marker,pre+marker,1)
start=s.index(marker);end=s.index('            store.stop_completed = true;',start)
chunk=s[start:end]
old='store.routes.as_mut().ok_or_else(||refused("原路由pin已转移或缺席"))?';assert chunk.count(old)==1
chunk=chunk.replace(old,'store.route_close.as_mut().ok_or_else(||refused("原路由关闭槽缺席"))?')
assert chunk.count('            store.check_source(owner, end)?;')==1
chunk=chunk.replace('            store.check_source(owner, end)?;','            store.source.check(owner,end)?;\n            store.core.journal().check_source_owner(owner,end).map_err(refused)?;\n            store.route_close.as_ref().ok_or_else(||refused("原路由关闭history缺席"))?.check_history(end).map_err(refused)?;')
s=s[:start]+chunk+s[end:]
marker='            store.failed = true;\n            store\n                .content_start\n                .check_completed_history('
assert s.count(marker)==1
s=s.replace(marker,'            store.failed = true;\n            if store.routes.is_some(){return Err(refused("完成历史仍持原live路由pin"));}\n            store.route_close.as_ref().ok_or_else(||refused("原路由关闭history缺席"))?.check_history(end).map_err(refused)?;\n            store.content_start.check_completed_history(',1)
p.write_text(s,encoding='utf-8',newline='')
print(json.dumps(dict(routeOption=True,retainedCloseSlotBeforeIO=True,actualGuardUsesCandidate=True)))
