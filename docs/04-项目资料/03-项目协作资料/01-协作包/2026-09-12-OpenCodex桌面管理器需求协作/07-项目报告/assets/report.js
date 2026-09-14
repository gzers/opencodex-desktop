/* OpenCodeX-Desktop 项目报告 · 共享导航
   职责：渲染顶栏与侧栏、页面切换、顶栏/侧栏布局切换、当前分区高亮。
   页面本身是静态 HTML，即使脚本不可用也能正常阅读。 */
(function () {
  var PAGES = [
    { file: "index.html",          label: "首页" },
    { file: "01-范围与能力.html",   label: "范围与能力" },
    { file: "02-架构与状态.html",   label: "架构与状态" },
    { file: "03-关键流程.html",     label: "关键流程" },
    { file: "04-数据与安全.html",   label: "数据与安全" },
    { file: "05-原型与设计.html",   label: "原型与设计" },
    { file: "06-进度与治理.html",   label: "进度与治理" },
    { file: "07-开工清单.html",     label: "开工清单" }
  ];
  var here = decodeURIComponent(location.pathname.split("/").pop() || "index.html");
  var idx = 0;
  for (var i = 0; i < PAGES.length; i++) { if (PAGES[i].file === here) idx = i; }

  /* 当前页内的分区 */
  var sections = [].slice.call(document.querySelectorAll("main [data-title]"));
  var subs = sections.map(function (s) {
    return { id: s.id, label: s.getAttribute("data-title") };
  });

  /* 顶栏 */
  var tb = document.getElementById("topbar");
  if (tb) {
    var h = '<div class="tb-in"><a class="brand" href="index.html"><i></i>OpenCodeX-Desktop 项目报告</a>';
    PAGES.forEach(function (p) {
      h += '<a class="np' + (p.file === here ? " on" : "") + '" href="' + p.file + '">' + p.label + "</a>";
    });
    h += '<button class="lt" id="lt" type="button" title="切换顶栏 / 侧栏导航">▤ 侧栏</button></div>';
    tb.innerHTML = h;
  }

  /* 侧栏 */
  var sd = document.getElementById("side");
  if (sd) {
    var s = '<div class="sd-brand"><i></i>OpenCodeX-Desktop</div><div class="sd-group">报告分册</div>';
    PAGES.forEach(function (p) {
      s += '<a class="' + (p.file === here ? "on" : "") + '" href="' + p.file + '">' + p.label + "</a>";
    });
    if (subs.length) {
      s += '<div class="sd-group">本页分区</div>';
      subs.forEach(function (x) {
        s += '<a class="sub" href="#' + x.id + '">' + x.label + "</a>";
      });
    }
    s += '<div class="sd-foot">依据 <span class="mono">COL-LOCAL-20260912-01</span> 与 <span class="mono">docs/</span> 当前事实<br>生成于 2026-09-14<br>派生产物，不替代 DMD</div>';
    s += '<div style="padding:10px 10px 0"><button class="lt" id="lt2" type="button">▤ 顶栏</button></div>';
    sd.innerHTML = s;
  }

  /* 布局切换 */
  var KEY = "ocx-report-layout";
  function apply(mode) {
    document.body.classList.toggle("mode-side", mode === "side");
    try { localStorage.setItem(KEY, mode); } catch (e) {}
  }
  var saved = "top";
  try { saved = localStorage.getItem(KEY) || "top"; } catch (e) {}
  apply(saved);
  function bind(id) {
    var b = document.getElementById(id);
    if (b) b.addEventListener("click", function () {
      apply(document.body.classList.contains("mode-side") ? "top" : "side");
    });
  }
  bind("lt"); bind("lt2");

  /* 当前分区高亮（侧栏） */
  if (subs.length) {
    var links = {};
    [].slice.call(document.querySelectorAll("#side a.sub")).forEach(function (a) {
      links[a.getAttribute("href").slice(1)] = a;
    });
    function sync() {
      var y = window.scrollY + 120, cur = subs[0].id;
      subs.forEach(function (x) {
        var el = document.getElementById(x.id);
        if (el && el.offsetTop <= y) cur = x.id;
      });
      Object.keys(links).forEach(function (k) { links[k].classList.remove("on"); });
      if (links[cur]) links[cur].classList.add("on");
    }
    window.addEventListener("scroll", sync, { passive: true });
    sync();
  }
})();
