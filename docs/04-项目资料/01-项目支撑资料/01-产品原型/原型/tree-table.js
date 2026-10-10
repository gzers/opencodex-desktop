/* 原型共享树表行组件。只生成标记；业务选择、展开和事务由页面适配器持有。 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.PrototypeTreeTable = api;
})(typeof window === 'undefined' ? globalThis : window, function () {
  'use strict';
  const escape = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  // HTML slots and attributes are trusted adapter output; names and IDs are escaped here.
  function name(options = {}) {
    const depth = Math.max(0, Number.isInteger(options.depth) ? options.depth : 0);
    const toggle = options.toggleAttrs
      ? '<button type="button" class="proto-tree-toggle ' + escape(options.toggleClass || '') + '" ' + options.toggleAttrs + ' aria-expanded="' + Boolean(options.expanded) + '" aria-label="' + escape((options.expanded ? '收起 ' : '展开 ') + options.label) + '"><span class="proto-tree-chevron" aria-hidden="true"></span></button>'
      : '<span class="proto-tree-spacer" aria-hidden="true"></span>';
    return '<div class="proto-tree-cell ' + escape(options.nameClass || '') + '" style="--tree-level:' + depth + '">' + (options.selection || '') + toggle + (options.icon || '') + '<div class="proto-tree-text">' + (options.nameHtml ?? escape(options.label)) + '</div></div>';
  }
  function row(options) {
    const table = options.table === true, tag = table ? 'tr' : (options.tag || 'div');
    const cells = (options.cells || []).map(html => table ? '<td>' + html + '</td>' : html).join('');
    const first = name(options);
    return '<' + tag + ' class="proto-tree-row ' + escape(options.className || '') + '" ' + (options.attrs || '') + '>' + (table ? '<th scope="row">' + first + '</th>' : first) + cells + '</' + tag + '>';
  }
  function visible(nodes, collapsed) {
    const byId = new Map(nodes.map(node => [node.id, node]));
    if (byId.size !== nodes.length) throw new Error('树表节点 ID 重复');
    return nodes.filter(node => {
      const visited = new Set([node.id]);
      let parent = node.parent;
      let hidden = false;
      while (parent) {
        if (visited.has(parent)) throw new Error('树表父节点循环');
        visited.add(parent);
        if (!byId.has(parent)) throw new Error('树表父节点缺失');
        if (collapsed.has(parent)) hidden = true;
        parent = byId.get(parent).parent;
      }
      return !hidden;
    });
  }
  function rows(nodes, adapter) { return nodes.map((node, index) => row(adapter(node, index))).join(''); }
  function details(options) {
    return '<details class="mp-details mp-draft-group" ' + (options.open ? 'open' : '') + '><summary>' + name({label: options.label, nameHtml: options.nameHtml}) + '</summary>' + options.body + '</details>';
  }
  return {name, row, rows, visible, details};
});
