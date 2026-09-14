/* 侧栏标题跳转 + 滚动高亮。纯前端，无依赖；脚本不可用时链接仍能跳转。 */
(function () {
  var links = [].slice.call(document.querySelectorAll('.side-nav a[href^="#"]'));
  if (!links.length) return;
  var map = {};
  links.forEach(function (a) { map[a.getAttribute('href').slice(1)] = a; });
  var targets = links.map(function (a) { return document.getElementById(a.getAttribute('href').slice(1)); })
                     .filter(Boolean);
  function sync() {
    var y = window.scrollY + 120, cur = targets.length ? targets[0].id : null;
    targets.forEach(function (t) { if (t.offsetTop <= y) cur = t.id; });
    if (window.innerHeight + window.scrollY >= document.body.scrollHeight - 4) {
      cur = targets[targets.length - 1].id;
    }
    links.forEach(function (a) { a.classList.remove('active'); });
    if (cur && map[cur]) map[cur].classList.add('active');
  }
  window.addEventListener('scroll', sync, { passive: true });
  window.addEventListener('resize', sync);
  sync();
})();
