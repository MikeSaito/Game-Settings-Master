(function(){const t=document.createElement("link").relList;if(t&&t.supports&&t.supports("modulepreload"))return;for(const n of document.querySelectorAll('link[rel="modulepreload"]'))r(n);new MutationObserver(n=>{for(const o of n)if(o.type==="childList")for(const l of o.addedNodes)l.tagName==="LINK"&&l.rel==="modulepreload"&&r(l)}).observe(document,{childList:!0,subtree:!0});function i(n){const o={};return n.integrity&&(o.integrity=n.integrity),n.referrerPolicy&&(o.referrerPolicy=n.referrerPolicy),n.crossOrigin==="use-credentials"?o.credentials="include":n.crossOrigin==="anonymous"?o.credentials="omit":o.credentials="same-origin",o}function r(n){if(n.ep)return;n.ep=!0;const o=i(n);fetch(n.href,o)}})();const m=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 800" fill="none" preserveAspectRatio="xMidYMid slice">
  <defs>
    <linearGradient id="sky" x1="600" y1="0" x2="600" y2="520">
      <stop offset="0%" stop-color="#0a1628"/>
      <stop offset="45%" stop-color="#121f35"/>
      <stop offset="100%" stop-color="#1a2840"/>
    </linearGradient>
    <linearGradient id="skyGlow" x1="400" y1="80" x2="900" y2="400">
      <stop offset="0%" stop-color="#5b8def" stop-opacity="0.18"/>
      <stop offset="100%" stop-color="#5b8def" stop-opacity="0"/>
    </linearGradient>
    <linearGradient id="water" x1="600" y1="480" x2="600" y2="800">
      <stop offset="0%" stop-color="#1e3a5f" stop-opacity="0.5"/>
      <stop offset="100%" stop-color="#0f1115"/>
    </linearGradient>
    <linearGradient id="peakNear" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#2a3f5c"/>
      <stop offset="100%" stop-color="#151921"/>
    </linearGradient>
    <linearGradient id="peakFar" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#3d5270"/>
      <stop offset="100%" stop-color="#1a2433"/>
    </linearGradient>
  </defs>

  <rect width="1200" height="800" fill="url(#sky)"/>
  <ellipse cx="620" cy="280" rx="420" ry="200" fill="url(#skyGlow)"/>

  <g id="layer-coarse" opacity="1">
    <ellipse cx="200" cy="120" rx="90" ry="90" fill="#eef1f6" opacity="0.07"/>
    <path d="M 0 520 Q 200 380 400 460 T 800 420 T 1200 480 L 1200 800 L 0 800 Z" fill="#151921" opacity="0.85"/>
    <path d="M 0 560 Q 300 440 600 500 T 1200 520 L 1200 800 L 0 800 Z" fill="#0f141c" opacity="0.9"/>
    <rect y="520" width="1200" height="280" fill="url(#water)" opacity="0.4"/>
    <circle cx="950" cy="140" r="50" fill="#5b8def" opacity="0.06"/>
  </g>

  <g id="layer-medium" opacity="0.2">
    <path d="M 0 480 L 180 320 L 320 400 L 480 260 L 640 360 L 820 220 L 980 340 L 1200 300 L 1200 800 L 0 800 Z" fill="url(#peakFar)" opacity="0.7"/>
    <path d="M 0 540 L 240 400 L 420 480 L 580 360 L 760 440 L 940 320 L 1200 400 L 1200 800 L 0 800 Z" fill="url(#peakNear)" opacity="0.85"/>
    <path d="M 0 620 Q 400 580 600 590 T 1200 610" stroke="#5b8def" stroke-width="1" opacity="0.25" fill="none"/>
    <g opacity="0.35">
      <path d="M 80 520 L 100 460 L 120 520 Z" fill="#1a2433"/>
      <path d="M 140 530 L 158 478 L 176 530 Z" fill="#1a2433"/>
      <path d="M 200 525 L 220 465 L 240 525 Z" fill="#212733"/>
      <path d="M 900 515 L 918 455 L 936 515 Z" fill="#1a2433"/>
      <path d="M 960 522 L 978 468 L 996 522 Z" fill="#212733"/>
      <path d="M 1020 518 L 1040 460 L 1060 518 Z" fill="#1a2433"/>
    </g>
    <circle cx="180" cy="95" r="2" fill="#eef1f6" opacity="0.5"/>
    <circle cx="320" cy="60" r="1.5" fill="#eef1f6" opacity="0.4"/>
    <circle cx="890" cy="80" r="2" fill="#eef1f6" opacity="0.45"/>
    <circle cx="1020" cy="110" r="1.5" fill="#eef1f6" opacity="0.35"/>
  </g>

  <g id="layer-fine" opacity="0">
    <path d="M 480 260 L 500 200 L 520 240 L 540 180 L 560 260 Z" fill="#c4cad4" opacity="0.15"/>
    <path d="M 820 220 L 838 165 L 856 210 L 874 175 L 892 220 Z" fill="#eef1f6" opacity="0.12"/>
    <path d="M 0 590 Q 200 570 400 575 T 800 568 T 1200 580" stroke="#5b8def" stroke-width="0.75" opacity="0.4" fill="none"/>
    <path d="M 350 595 Q 600 585 850 592" stroke="#7eb0ff" stroke-width="0.5" opacity="0.25" fill="none"/>
    <g stroke="#5b8def" stroke-width="0.5" opacity="0.2">
      <line x1="100" y1="640" x2="180" y2="640"/>
      <line x1="220" y1="655" x2="320" y2="655"/>
      <line x1="880" y1="645" x2="980" y2="645"/>
    </g>
    <text x="48" y="750" fill="#727b8a" font-family="JetBrains Mono, monospace" font-size="10" opacity="0.5">focus ∞</text>
    <text x="1050" y="750" fill="#5b8def" font-family="JetBrains Mono, monospace" font-size="10" opacity="0.35">clarity</text>
    <circle cx="200" cy="95" r="3" fill="#eef1f6" opacity="0.6"/>
    <circle cx="320" cy="60" r="2" fill="#eef1f6" opacity="0.5"/>
    <circle cx="450" cy="45" r="1.5" fill="#eef1f6" opacity="0.4"/>
    <circle cx="720" cy="70" r="2" fill="#eef1f6" opacity="0.45"/>
    <circle cx="890" cy="80" r="2.5" fill="#eef1f6" opacity="0.55"/>
    <circle cx="1020" cy="110" r="2" fill="#eef1f6" opacity="0.4"/>
    <path d="M 600 120 Q 620 100 640 120" stroke="#5b8def" stroke-width="1" opacity="0.2" fill="none"/>
  </g>
</svg>
`,d=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 200" fill="none">
  <rect width="280" height="200" rx="16" fill="#151921" stroke="#2f3642" stroke-width="1"/>
  <rect x="20" y="28" width="72" height="96" rx="8" fill="#1a1f28" stroke="#434b58"/>
  <rect x="30" y="78" width="52" height="6" rx="2" fill="#434b58"/>
  <rect x="30" y="92" width="40" height="5" rx="2" fill="#2f3642"/>
  <rect x="104" y="28" width="72" height="96" rx="8" fill="#1a1f28" stroke="#5b8def" stroke-width="1.5"/>
  <rect x="114" y="78" width="52" height="6" rx="2" fill="#5b8def" opacity="0.55"/>
  <rect x="188" y="28" width="72" height="96" rx="8" fill="#1a1f28" stroke="#434b58"/>
  <circle cx="224" cy="52" r="14" stroke="#727b8a" stroke-width="1.5" fill="none"/>
  <path d="M 218 52 L 223 57 L 232 48" stroke="#5b8def" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
  <g transform="translate(140 162)">
    <circle r="14" stroke="#5b8def" stroke-width="2.5" fill="rgba(91,141,239,0.12)"/>
    <path d="M 9.9 9.9 L 21 21" stroke="#5b8def" stroke-width="2.5" stroke-linecap="round"/>
  </g>
</svg>
`,f=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 200" fill="none">
  <rect x="36" y="28" width="208" height="34" rx="8" fill="#1a1f28" stroke="#434b58"/>
  <rect x="36" y="74" width="208" height="34" rx="8" fill="#1a1f28" stroke="#434b58"/>
  <rect x="36" y="120" width="208" height="40" rx="8" fill="rgba(91,141,239,0.12)" stroke="#5b8def" stroke-width="1.5"/>
  <rect x="52" y="42" width="56" height="6" rx="2" fill="#727b8a"/>
  <rect x="52" y="88" width="72" height="6" rx="2" fill="#727b8a"/>
  <rect x="52" y="137" width="88" height="6" rx="2" fill="#5b8def"/>
  <path d="M 214 42 L 220 48 L 228 38" stroke="#727b8a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M 214 88 L 220 94 L 228 84" stroke="#727b8a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M 214 137 L 220 143 L 228 133" stroke="#5b8def" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M 196 18 L 204 10 L 210 18 L 220 8" stroke="#5b8def" stroke-width="2" stroke-linecap="round" fill="none"/>
  <circle cx="220" cy="8" r="3" fill="#5b8def"/>
</svg>
`,p=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 200" fill="none">
  <rect x="60" y="50" width="160" height="100" rx="12" fill="#1a1f28" stroke="#434b58" stroke-width="1.5"/>
  <rect x="80" y="70" width="120" height="60" rx="6" fill="#0f1115" stroke="#2f3642"/>
  <rect x="95" y="85" width="24" height="30" rx="2" fill="#212733" stroke="#434b58"/>
  <rect x="128" y="85" width="24" height="30" rx="2" fill="#212733" stroke="#434b58"/>
  <rect x="161" y="85" width="24" height="30" rx="2" fill="#212733" stroke="#434b58"/>
  <text x="98" y="105" fill="#5b8def" font-size="7" font-family="monospace">DLSS</text>
  <text x="134" y="105" fill="#727b8a" font-size="7" font-family="monospace">RT</text>
  <text x="166" y="105" fill="#5b8def" font-size="7" font-family="monospace">FG</text>
  <rect x="100" y="165" width="80" height="8" rx="4" fill="#212733"/>
  <rect x="100" y="165" width="56" height="8" rx="4" fill="#5b8def" opacity="0.7"/>
  <circle cx="140" cy="40" r="12" fill="rgba(91,141,239,0.2)" stroke="#5b8def"/>
  <path d="M 136 40 L 140 44 L 148 34" stroke="#5b8def" stroke-width="1.5" stroke-linecap="round"/>
</svg>
`,h=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 200" fill="none">
  <rect x="28" y="20" width="224" height="160" rx="12" fill="#151921" stroke="#434b58"/>
  <text x="44" y="48" fill="#5b8def" font-family="JetBrains Mono, monospace" font-size="10">[SystemSettings]</text>
  <text x="44" y="72" fill="#9aa3b2" font-family="JetBrains Mono, monospace" font-size="9">r.ViewDistanceScale</text>
  <rect x="44" y="78" width="160" height="6" rx="3" fill="#212733"/>
  <rect x="44" y="78" width="112" height="6" rx="3" fill="#5b8def" opacity="0.65"/>
  <circle cx="156" cy="81" r="7" fill="#1a1f28" stroke="#5b8def" stroke-width="1.5"/>
  <circle cx="156" cy="81" r="2.5" fill="#5b8def"/>
  <text x="44" y="106" fill="#9aa3b2" font-family="JetBrains Mono, monospace" font-size="9">r.Shadow.MaxResolution</text>
  <rect x="44" y="112" width="160" height="6" rx="3" fill="#212733"/>
  <rect x="44" y="112" width="64" height="6" rx="3" fill="#727b8a"/>
  <text x="44" y="140" fill="#727b8a" font-family="JetBrains Mono, monospace" font-size="9">sg.PostProcessQuality</text>
  <rect x="44" y="146" width="160" height="6" rx="3" fill="#212733"/>
  <rect x="44" y="146" width="120" height="6" rx="3" fill="#5b8def" opacity="0.5"/>
</svg>
`,u=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 200" fill="none">
  <path d="M 140 44 C 98 44 68 74 68 112 C 68 150 98 172 140 172 C 182 172 212 150 212 112" stroke="#434b58" stroke-width="3" stroke-linecap="round" fill="none"/>
  <path d="M 128 54 L 140 38 L 152 54" stroke="#5b8def" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
  <circle cx="140" cy="112" r="38" stroke="#5b8def" stroke-width="1" fill="rgba(91,141,239,0.06)" opacity="0.45"/>
  <rect x="108" y="96" width="64" height="48" rx="8" fill="#1a1f28" stroke="#5b8def" stroke-width="1.5"/>
  <path d="M 122 116 L 134 128 L 158 104" stroke="#5b8def" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  <text x="140" y="168" fill="#727b8a" font-size="9" font-family="JetBrains Mono, monospace" text-anchor="middle">snapshot</text>
</svg>
`,y=`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 200" fill="none">
  <path d="M 88 88 C 72 88 64 100 68 112 C 56 116 52 128 60 140 C 68 152 84 156 100 148 L 180 148 C 200 148 212 132 208 118 C 212 102 198 88 180 88 C 176 72 154 64 138 72 C 128 64 108 64 100 76 C 92 80 88 84 88 88 Z" fill="rgba(91,141,239,0.1)" stroke="#5b8def" stroke-width="1.5"/>
  <path d="M 140 108 L 140 128" stroke="#5b8def" stroke-width="2" stroke-linecap="round"/>
  <path d="M 134 122 L 140 128 L 146 122" stroke="#5b8def" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
  <rect x="100" y="132" width="80" height="36" rx="8" fill="#1a1f28" stroke="#434b58"/>
  <text x="140" y="154" fill="#9aa3b2" font-size="8" font-family="JetBrains Mono, monospace" text-anchor="middle">catalog.json</text>
  <circle cx="198" cy="96" r="3.5" fill="#5b8def" opacity="0.55"/>
  <circle cx="210" cy="104" r="2.5" fill="#5b8def" opacity="0.4"/>
  <circle cx="188" cy="108" r="2" fill="#5b8def" opacity="0.35"/>
</svg>
`,g={lang:"en",htmlLang:"en",siteName:"Game Settings Master",meta:{title:"Game Settings Master — graphics presets for games",description:"Presets, manual editor and cloud sync for Unreal Engine, Unity and author-curated game breakdowns.",keywords:"game settings, graphics presets, Unreal Engine, Unity, DLSS, FSR, config editor",ogLocale:"en_US"},nav:{features:"Features",download:"Download",aria:"Navigation"},hero:{badge:"Game Settings Master",title:"Game settings",titleAccent:"in focus",subtitle:"Graphics master for Unreal Engine, Unity and author-curated breakdowns — without digging through configs manually."},engineTags:["UE 4","UE 5","Unity","Author-curated"],features:[{id:"library",step:"01",title:"Game library",text:"Steam and Epic scan, manual add. The app finds your config folder automatically.",illustration:d},{id:"presets",step:"02",title:"One-click presets",text:"From Ultra Low to Ultra High — with a diff preview before apply. Every change visible in configs.",illustration:f,reverse:!0},{id:"smart",step:"03",title:"Smart tuning",text:"DLSS, FSR, ray tracing and Frame Generation — safe clamp for your GPU. No pointless options on weak hardware.",illustration:p},{id:"editor",step:"04",title:"Manual editor",text:"Over a hundred parameters with descriptions, categories and dependencies.",illustration:h,reverse:!0},{id:"backup",step:"05",title:"Backups",text:"Snapshot before every apply. Roll back to the previous state in one click — no fear of breaking your config.",illustration:u},{id:"cloud",step:"06",title:"Cloud presets",text:"Content syncs from the server without an app release. Offline — built-in cache fallback.",illustration:y,reverse:!0}],download:{title:"Download the app",subtitle:"Windows · free",button:"Download for Windows",soon:"Coming soon for Windows",soonHint:"Release in progress — stay tuned",soonTitle:"Release in progress"},footer:{version:e=>`Game Settings Master v${e}`},localeSwitch:{label:"Language",ru:"RU",en:"EN"}},w={lang:"ru",htmlLang:"ru",siteName:"Game Settings Master",meta:{title:"Game Settings Master — мастер графики для игр",description:"Пресеты, ручной редактор и облачная синхронизация настроек для Unreal Engine, Unity и авторских разборов.",keywords:"game settings, пресеты графики, Unreal Engine, Unity, настройки игр, DLSS, FSR",ogLocale:"ru_RU"},nav:{features:"Возможности",download:"Скачать",aria:"Навигация"},hero:{badge:"Game Settings Master",title:"Настройки игр",titleAccent:"в фокусе",subtitle:"Мастер графики для Unreal Engine, Unity и авторских разборов других игр — без ручного ковыряния в конфигах."},engineTags:["UE 4","UE 5","Unity","Авторские разборы"],features:[{id:"library",step:"01",title:"Библиотека игр",text:"Сканирование Steam и Epic, ручное добавление. Приложение само находит папку конфигурации.",illustration:d},{id:"presets",step:"02",title:"Пресеты в один клик",text:"От Ultra Low до Ultra High — с предпросмотром diff до применения. Видно каждую правку в конфигах.",illustration:f,reverse:!0},{id:"smart",step:"03",title:"Умная настройка",text:"DLSS, FSR, ray tracing и Frame Generation — безопасный clamp под ваш GPU. Без бессмысленных опций на слабом железе.",illustration:p},{id:"editor",step:"04",title:"Ручной редактор",text:"Более сотни параметров с описаниями, категориями и зависимостями.",illustration:h,reverse:!0},{id:"backup",step:"05",title:"Бэкапы",text:"Snapshot перед каждым apply. Откат к предыдущему состоянию одним кликом — без страха сломать конфиг.",illustration:u},{id:"cloud",step:"06",title:"Облачные пресеты",text:"Контент с сервера синхронизируется без релиза приложения. Offline — встроенный fallback из кэша.",illustration:y,reverse:!0}],download:{title:"Скачать приложение",subtitle:"Windows · бесплатно",button:"Скачать для Windows",soon:"Скоро в Windows",soonHint:"Релиз готовится — следите за обновлениями",soonTitle:"Релиз готовится"},footer:{version:e=>`Game Settings Master v${e}`},localeSwitch:{label:"Язык",ru:"RU",en:"EN"}},a={ru:w,en:g};function b(){const e=document.documentElement.lang;return e&&a[e]?a[e]:/^\/en(?:\/|\.html)?$/.test(window.location.pathname)?g:w}function k(e){const t=document.createElement("header");t.className="site-header";const i=e.lang==="en"?"/en/":"/",r=e.lang==="en"?"/":"/en/",n=e.lang==="en"?e.localeSwitch.ru:e.localeSwitch.en;t.innerHTML=`
    <a href="${i}" class="site-header__brand">
      <img src="/logo.png" width="28" height="28" alt="" class="site-header__logo" />
      <span>${e.siteName}</span>
    </a>
    <nav class="site-header__nav" aria-label="${e.nav.aria}">
      <a href="${i}#features">${e.nav.features}</a>
      <a href="${i}#download">${e.nav.download}</a>
      <a href="${r}" class="site-header__locale" hreflang="${e.lang==="en"?"ru":"en"}">${n}</a>
    </nav>
  `;const o=()=>{t.classList.toggle("is-scrolled",window.scrollY>50)};return window.addEventListener("scroll",o,{passive:!0}),o(),t}function L(e){const t=document.createElement("section");return t.className="hero page-wrap",t.innerHTML=`
    <div class="hero__badge">${e.hero.badge}</div>
    <h1 class="hero__title">${e.hero.title} <span>${e.hero.titleAccent}</span></h1>
    <p class="hero__subtitle">${e.hero.subtitle}</p>
    <div class="hero__engines">
      ${e.engineTags.map(i=>`<span class="engine-tag">${i}</span>`).join("")}
    </div>
  `,t}function v(e){const t=document.createElement("article");t.className=`feature${e.reverse?" feature--reverse":""}`,t.id=e.id,t.innerHTML=`
    <div class="feature__content">
      <div class="feature__step">${e.step}</div>
      <h2 class="feature__title">${e.title}</h2>
      <p class="feature__text">${e.text}</p>
    </div>
    <div class="feature__illus" aria-hidden="true">${e.illustration}</div>
  `;const i=new IntersectionObserver(([r])=>{r?.isIntersecting&&(t.classList.add("is-visible"),i.disconnect())},{threshold:.2,rootMargin:"0px 0px -10% 0px"});return i.observe(t),t}const x="0.1.0";function M(e){const t=document.createElement("section");return t.className="download page-wrap",t.id="download",t.innerHTML=`
    <div class="download__card">
      <h2 class="download__title">${e.download.title}</h2>
      <p class="download__text">${e.download.subtitle} · v${x}</p>
      <div class="download__engines">
        ${e.engineTags.map(i=>`<span class="engine-tag">${i}</span>`).join("")}
      </div>
      ${`<button class="btn btn--primary" type="button" disabled title="${e.download.soonTitle}">${e.download.soon}</button>
             <span class="btn__hint">${e.download.soonHint}</span>`}
    </div>
  `,t}function S(e){const t=document.createElement("footer");return t.className="site-footer",t.innerHTML=`<p>${e.footer.version(x)}</p>`,t}function s(e,t,i){return e+(t-e)*i}let c=-1;function E(e){if(Math.abs(e-c)<.004)return;c=e;const t=document.documentElement,i=s(1.07,1,e);t.style.setProperty("--reveal",String(e)),t.style.setProperty("--scene-scale",String(i));const r=document.getElementById("layer-coarse"),n=document.getElementById("layer-medium"),o=document.getElementById("layer-fine");r&&r.setAttribute("opacity",String(s(1,.15,e))),n&&n.setAttribute("opacity",String(s(.12,.75,e))),o&&o.setAttribute("opacity",String(s(0,1,e)))}function _(e){const t=document.getElementById("scene-root");t&&(t.innerHTML=e)}function $(){const t=document.documentElement.scrollHeight-window.innerHeight;return t<=0?1:Math.min(1,Math.max(0,window.scrollY/t))}function G(){return window.matchMedia("(prefers-reduced-motion: reduce)").matches}function U(e){let t=!1;const i=()=>{t=!1;const l=G()?1:$();e(l)},r=()=>{t||(t=!0,requestAnimationFrame(i))},n=window.matchMedia("(prefers-reduced-motion: reduce)"),o=()=>i();return window.addEventListener("scroll",r,{passive:!0}),window.addEventListener("resize",r,{passive:!0}),n.addEventListener("change",o),i(),()=>{window.removeEventListener("scroll",r),window.removeEventListener("resize",r),n.removeEventListener("change",o)}}function B(){const e=b();_(m);const t=document.getElementById("app");if(!t)return;const i=document.createElement("main");i.className="site-main";const r=document.createElement("section");r.className="features page-wrap",r.id="features";for(const n of e.features)r.appendChild(v(n));i.append(L(e),r,M(e),S(e)),t.append(k(e),i),U(E)}B();
