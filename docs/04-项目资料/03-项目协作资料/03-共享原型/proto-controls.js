/* 原型控制器：锚点栏（tab 式跳转）+ 当前分区高亮。
   用法：给分区容器加 id，并在 .pctl-bar 里放 <a href="#id">分区名</a>（也支持 <button data-pctl-jump="id">）。
   滚动容器自动判定：优先用最近的可滚动祖先，否则退回窗口。 */
(()=>{
  function scrollerFor(el){
    let node=el.parentElement;
    while(node&&node!==document.body){
      const style=getComputedStyle(node);
      if(/(auto|scroll)/.test(style.overflowY)&&node.scrollHeight>node.clientHeight+4) return node;
      node=node.parentElement;
    }
    return null;
  }
  function offsetWithin(scroller,el){
    if(!scroller) return window.scrollY+el.getBoundingClientRect().top;
    const sr=scroller.getBoundingClientRect(),er=el.getBoundingClientRect();
    return scroller.scrollTop+(er.top-sr.top);
  }
  function scrollToEl(el,smooth){
    const scroller=scrollerFor(el);
    const top=Math.max(0,offsetWithin(scroller,el)-12);
    if(scroller) scroller.scrollTo({top,behavior:smooth?'smooth':'auto'});
    else window.scrollTo({top,behavior:smooth?'smooth':'auto'});
  }

  document.querySelectorAll('.pctl-bar').forEach(bar=>{
    const links=[...bar.querySelectorAll('a[href^="#"],button[data-pctl-jump]')];
    if(!links.length) return;
    const targets=links.map(l=>{
      const id=(l.dataset.pctlJump||l.getAttribute('href')||'').replace(/^#/,'');
      return id?document.getElementById(id):null;
    });

    let lockUntil=0; // 点击后短暂锁定高亮，避免「滑到底部被最后一节抢走」
    links.forEach((link,i)=>{
      link.addEventListener('click',event=>{
        const target=targets[i];
        if(!target) return;
        event.preventDefault();
        links.forEach(other=>other.classList.toggle('active',other===link));
        target.setAttribute('data-pctl-anchor','');
        scrollToEl(target,true);
        history.replaceState(null,'','#'+target.id);
        lockUntil=performance.now()+1600;
      });
    });

    const scroller=targets.find(Boolean)?scrollerFor(targets.find(Boolean)):null;
    const host=scroller||window;
    let ticking=false;
    const sync=()=>{
      ticking=false;
      if(performance.now()<lockUntil) return;
      const pos=scroller?scroller.scrollTop:window.scrollY;
      const maxScroll=scroller?scroller.scrollHeight-scroller.clientHeight
        :document.documentElement.scrollHeight-window.innerHeight;
      let best=-1,bestTop=-Infinity;
      targets.forEach((t,i)=>{
        if(!t) return;
        const top=offsetWithin(scroller,t);
        if(top-22<=pos&&top>bestTop){bestTop=top;best=i}
      });
      // 触底：最后一节到不了自己的触发线，直接算它当前。
      if(pos>=maxScroll-2){for(let i=targets.length-1;i>=0;i--){if(targets[i]){best=i;break}}}
      if(best<0) best=0;
      links.forEach((l,i)=>l.classList.toggle('active',i===best));
    };
    host.addEventListener('scroll',()=>{if(!ticking){ticking=true;requestAnimationFrame(sync)}},{passive:true});
    sync();
  });
})();
