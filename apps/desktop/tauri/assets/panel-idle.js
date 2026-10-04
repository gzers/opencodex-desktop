// Only reclaim a clean, hidden view. Never stops or restarts the official proxy.
(() => {
  let edited = false;
  const markEdited = () => { edited = true; };
  document.addEventListener('input', markEdited, true);
  document.addEventListener('change', markEdited, true);
  // Conservatively retain the view after any edit: the official page supplies
  // no save acknowledgement contract, so a guessed "saved" state risks data loss.
  window.__ocxdTryReclaim = (generation) => {
    if (edited || document.querySelector('[contenteditable="true"]:focus')) return;
    const query = new URLSearchParams({ generation: String(generation), path: location.pathname, fragment: location.hash.slice(1), scroll: String(scrollY) });
    location.href = 'ocxd-panel://reclaim?' + query;
  };
})();
