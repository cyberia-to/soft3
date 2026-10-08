// soft3 — one document, many pages: internal links swap the shell and keep the world.
// the canvas never reloads, the panels cross-fade (View Transitions where the browser has them).
(() => {
  const INTERNAL = (u) => u.origin === location.origin && /^\/(chart\/|loop\/|scheme\/)?$/.test(u.pathname);
  const cache = new Map();
  const fetchPage = (href) => cache.get(href) || (cache.set(href, fetch(href, { credentials: "same-origin" }).then((r) => r.text())), cache.get(href));
  function swapTo(html, href, push) {
    const doc = new DOMParser().parseFromString(html, "text/html");
    const shell = doc.querySelector(".shell"); if (!shell) { location.href = href; return; }
    const style = [...doc.head.querySelectorAll("style")].map((s) => s.textContent).join("\n");
    const scripts = [...doc.querySelectorAll("script:not([src])")].map((s) => s.textContent);
    const apply = () => {
      const cur = document.querySelector(".shell");
      let ps = document.getElementById("page-style");
      if (!ps) { ps = document.createElement("style"); ps.id = "page-style"; document.head.appendChild(ps); }
      ps.textContent = style;
      cur.innerHTML = shell.innerHTML;
      document.title = doc.title;
      if (push) history.pushState({ soft3: true }, "", href);
      window.scrollTo(0, 0);
      for (const text of scripts) { const s = document.createElement("script"); s.textContent = text; document.body.appendChild(s); s.remove(); }
      dispatchEvent(new CustomEvent("soft3:page", { detail: { href } }));
    };
    if (document.startViewTransition) document.startViewTransition(apply); else { const cur = document.querySelector(".shell"); cur.style.transition = "opacity 180ms ease"; cur.style.opacity = "0"; setTimeout(() => { apply(); requestAnimationFrame(() => { cur.style.opacity = "1"; }); }, 180); }
  }
  const go = (href, push = true) => fetchPage(href).then((html) => swapTo(html, href, push)).catch(() => { location.href = href; });
  window.soft3Go = (href) => go(href);
  document.addEventListener("click", (e) => {
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    const a = e.target.closest("a[href]"); if (!a || a.target === "_blank") return;
    const u = new URL(a.href, location.href); if (!INTERNAL(u)) return;
    e.preventDefault(); go(u.pathname + u.search);
  });
  document.addEventListener("mouseover", (e) => { const a = e.target.closest("a[href]"); if (!a) return; const u = new URL(a.href, location.href); if (INTERNAL(u)) fetchPage(u.pathname + u.search); });
  addEventListener("popstate", () => go(location.pathname + location.search, false));
  history.replaceState({ soft3: true }, "", location.href);
})();
