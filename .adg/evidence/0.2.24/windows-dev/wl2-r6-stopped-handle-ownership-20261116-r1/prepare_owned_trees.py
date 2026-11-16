from pathlib import Path
import re,json
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
p=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/capture.rs'
text=p.read_text();assert '    trees: [AnchoredTree; 2],\n    journal: Journal,' in text
text=text.replace('    trees: [AnchoredTree; 2],\n    journal: Journal,','    trees: Option<[AnchoredTree; 2]>,\n    journal: Journal,',1)
cursor=0;pieces=[];changed=0
while True:
    start=text.find('        let result = (|| {',cursor)
    if start<0:pieces.append(text[cursor:]);break
    end=text.index('        })();',start)
    pieces.append(text[cursor:start])
    body=text[start:end]
    if re.search(r'self\s*\.trees',body):
        body=re.sub(r'&mut self\s*\.trees','trees',body)
        body=re.sub(r'self\s*\.trees','trees',body)
        body=body.replace('for tree in trees {','for tree in trees.iter_mut() {')
        body=body.replace('        let result = (|| {','        let result = (|| {\n            let trees = self.trees.as_mut().ok_or_else(invalid)?;',1)
        changed+=1
    pieces.append(body);cursor=end
text=''.join(pieces)
text=text.replace('            trees,\n            journal,','            trees: Some(trees),\n            journal,',1)
text=text.replace('        for tree in &mut self.trees {\n            tree.poisoned = true;\n        }','        if let Some(trees) = &mut self.trees {\n            for tree in trees { tree.poisoned = true; }\n        }',1)
text=text.replace('            self.trees\n                .iter()\n                .flat_map(|t| {','            self.trees\n                .iter().flatten()\n                .flat_map(|t| {',1)
assert changed==11,changed
p.write_text(text,encoding='utf-8',newline='')
p=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/capture_tests.rs'
text=p.read_text()
text=text.replace('&mut core.trees,','core.trees.as_mut().unwrap(),')
text=text.replace('core.trees[0]','core.trees.as_ref().unwrap()[0]')
text=text.replace('core.trees.iter()','core.trees.as_ref().unwrap().iter()')
p.write_text(text,encoding='utf-8',newline='')
print(json.dumps(dict(resultClosures=changed,ownedTreeSlot='Option exact two original trees',noNativeClone=True)))
