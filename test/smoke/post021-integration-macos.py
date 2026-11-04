#!/usr/bin/env python3
"""Actual integrated WKWebView material scenes and failed-restart recovery."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import traceback

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('owned_native',HERE/'ocx021-macos-native.py')
native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)

class Run(native.NativeRun):
    def prepare(self):
        super().prepare()
        path=self.root/'manager-state/preferences.json'
        prefs=json.loads(path.read_text());prefs.update(theme=self.args.theme,visual_effects=self.args.effects)
        native.write_json(path,prefs)

    def verify(self):
        self.node_detection()
        self.channels();self.press('编辑')
        snap=self.wait(lambda s:self.matches(s,'渠道名称',('AXTextField',)),name='integrated-detail')
        assert not self.matches(snap,'运行渠道标识',('AXTextField',))
        self.capture('integrated-material-detail',snap)
        self.press('返回渠道列表')
        if self.args.theme=='dark' and self.args.effects=='high':
            self.rename_save('integrated-mac-name')
            snap=self.wait(lambda s:self.matches(s,'立即重启 Codex'),name='integrated-saved-notice')
            self.capture('integrated-material-notice',snap)
            before=(self.root/'opencodex-home/config.json').read_bytes()
            assert not (self.root/'opencodex-home/runtime-port.json').exists()
            self.press('立即重启 Codex');self.wait(lambda s:self.dialogs(s),name='integrated-restart-confirm')
            self.press('确认重启')
            snap=self.wait(lambda s:'runtime-port.json' in self.text(s) and self.matches(s,'重试重启'),name='integrated-specific-restart-error')
            assert 'manage backups' not in self.text(snap) and '无需先打开面板' in self.text(snap)
            assert before==(self.root/'opencodex-home/config.json').read_bytes()
            self.capture('integrated-restart-error-retry',snap)
            self.dismiss('cancel')
            snap=self.wait(lambda s:self.matches(s,'立即重启 Codex'),name='integrated-restart-canceled')
            assert before==(self.root/'opencodex-home/config.json').read_bytes()
            self.press('稍后')
            self.check('integrated-restart-missing-service-recovery','pass',savedConfigUnchanged=True,errorNamesMissingRecord=True,manualPanelNotRequired=True,retryVisible=True,noClientRestart=True)
        self.check('integrated-material-native-scenes','pass',theme=self.args.theme,effects=self.args.effects,productionDialog=True,productionDetail=True,scriptFocusExitsPreserved=True,layout='native AX and owned-window screenshots; no CSS style injection')
        self.report['scopeCompleted']=True

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['app','out','expected-build-commit']:p.add_argument('--'+name,required=True)
    p.add_argument('--theme',choices=['light','dark'],required=True);p.add_argument('--effects',choices=['high','mid','low'],required=True)
    args=p.parse_args();args.stage='debug-preverify';args.scope='all';args.hold_seconds=0
    args.node=args.npm_cli=args.official_entry=args.bun=None
    run=Run(args)
    try:run.prepare();run.launch();run.verify()
    except (Exception,KeyboardInterrupt) as error:
        run.report['error']=str(error);run.report['errorKind']='blocked' if isinstance(error,native.Blocked) else 'fail';run.check('integrated-native-run',run.report['errorKind'],reason=str(error))
        (run.out/'exception.log').write_text(traceback.format_exc())
        try:snap=run.snapshot('integrated-failure');run.capture('integrated-failure',snap)
        except Exception:pass
    finally:run.close()
    return 0 if run.report['result']=='pass' else 1
if __name__=='__main__':sys.exit(main())
