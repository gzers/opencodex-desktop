const {test}=require('node:test');
const assert=require('node:assert/strict');
const tree=require('./tree-table.js');
test('父级折叠隐藏所有后代；再次展开保留孙级独立状态',()=>{
  const nodes=[{id:'root'},{id:'year',parent:'root'},{id:'month',parent:'year'},{id:'file',parent:'month'}];
  const collapsed=new Set(['month']);
  assert.deepEqual(tree.visible(nodes,collapsed).map(n=>n.id),['root','year','month']);
  collapsed.add('root');assert.deepEqual(tree.visible(nodes,collapsed).map(n=>n.id),['root']);
  collapsed.delete('root');assert.deepEqual(tree.visible(nodes,collapsed).map(n=>n.id),['root','year','month']);
  collapsed.delete('month');assert.equal(tree.visible(nodes,collapsed).length,4);
});
test('损坏目录关系明确拒绝，不无限遍历或静默重复',()=>{
  assert.throws(()=>tree.visible([{id:'a'},{id:'a'}],new Set()),/重复/);
  assert.throws(()=>tree.visible([{id:'a',parent:'unknown'}],new Set()),/缺失/);
  assert.throws(()=>tree.visible([{id:'a',parent:'b'},{id:'b',parent:'a'}],new Set()),/循环/);
  assert.throws(()=>tree.visible([{id:'a',parent:'b'},{id:'b',parent:'a'}],new Set(['a','b'])),/循环/);
  assert.throws(()=>tree.visible([{id:'child',parent:'root'},{id:'root',parent:'missing'}],new Set(['root'])),/缺失/);
});
test('名称与展开标签转义；表格和网格共用选择、箭头、图标、文字结构',()=>{
  const options={label:'<script>&"',depth:1,expanded:false,toggleAttrs:'data-toggle="a"',selection:'<input type="checkbox" disabled>',icon:'<svg></svg>',cells:['用途','操作']};
  const grid=tree.row(options),table=tree.row({...options,table:true});
  assert.ok(!grid.includes('<script>'));assert.ok(grid.includes('&lt;script&gt;&amp;&quot;'));
  assert.ok(grid.includes('aria-expanded="false"'));assert.ok(table.includes('<th scope="row">'+tree.name(options)+'</th>'));
  assert.equal(grid.indexOf('<input')<grid.indexOf('proto-tree-toggle'),true);
  assert.equal(grid.indexOf('proto-tree-toggle')<grid.indexOf('<svg'),true);
  assert.equal(grid.indexOf('<svg')<grid.indexOf('proto-tree-text'),true);
  assert.ok(!tree.row({label:'leaf'}).includes('aria-expanded'));
});
