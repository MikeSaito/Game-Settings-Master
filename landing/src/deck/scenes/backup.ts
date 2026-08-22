import type { LocaleStrings } from "../../i18n/types";
import { buildAppWindow } from "../../mock/appWindow";
import { gsap, DESKTOP_QUERY, type Scene } from "../scroll";
import { sceneShell } from "./shell";

const SVG_NS = "http://www.w3.org/2000/svg";

function layoutCenter(el: HTMLElement, container: HTMLElement): { x: number; y: number } {
  let x = 0;
  let y = 0;
  let node: HTMLElement | null = el;
  while (node && node !== container) {
    x += node.offsetLeft;
    y += node.offsetTop;
    node = node.offsetParent as HTMLElement | null;
  }
  return { x: x + el.offsetWidth / 2, y: y + el.offsetHeight / 2 };
}

function buildShield(): { svg: SVGSVGElement; paths: SVGPathElement[] } {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("class", "backup__shield");
  svg.setAttribute("viewBox", "0 0 48 48");
  svg.setAttribute("aria-hidden", "true");

  const body = document.createElementNS(SVG_NS, "path");
  body.setAttribute("class", "backup__shield-body");
  body.setAttribute("d", "M24 4 L40 10 V22 C40 33 33 41 24 44 C15 41 8 33 8 22 V10 Z");

  const check = document.createElementNS(SVG_NS, "path");
  check.setAttribute("class", "backup__shield-check");
  check.setAttribute("d", "M16 24 L22 30 L33 18");

  svg.append(body, check);
  return { svg: svg as SVGSVGElement, paths: [body, check] };
}

export function buildBackupScene(t: LocaleStrings): Scene {
  const shell = sceneShell("backup", t.story.backup);

  const wrap = document.createElement("div");
  wrap.className = "backup";

  const winWrap = document.createElement("div");
  winWrap.className = "backup__win";
  const app = buildAppWindow(t, { initial: "backups" });
  winWrap.append(app.el);

  const list = app.el.querySelector<HTMLElement>(".mock__list");

  const newItem = document.createElement("li");
  newItem.className = "backup__item";
  const card = document.createElement("div");
  card.className = "mock__backup mock__backup--new";
  const info = document.createElement("div");
  info.className = "mock__backup-info";
  const topLine = document.createElement("div");
  topLine.className = "mock__backup-top";
  const label = document.createElement("span");
  label.className = "mock__backup-label";
  label.textContent = t.story.backup.now;
  const idEl = document.createElement("span");
  idEl.className = "mock__backup-id";
  idEl.textContent = t.mock.newBackupId;
  topLine.append(label, idEl);
  const files = document.createElement("div");
  files.className = "mock__backup-files";
  for (const file of t.mock.backups[0]?.files ?? ["GameUserSettings.ini"]) {
    const chip = document.createElement("span");
    chip.className = "mock__badge mock__badge--neutral";
    chip.textContent = file;
    files.append(chip);
  }
  info.append(topLine, files);
  const badge = document.createElement("span");
  badge.className = "backup__badge";
  badge.textContent = t.story.backup.badge;
  const shield = buildShield();
  card.append(info, badge, shield.svg);
  newItem.append(card);
  if (list) list.prepend(newItem);

  const fly = document.createElement("div");
  fly.className = "backup__fly";
  fly.setAttribute("aria-hidden", "true");
  fly.textContent = "GameUserSettings.ini";

  wrap.append(winWrap, fly);
  shell.stage.append(wrap);

  return {
    el: shell.section,
    init(mm) {
      mm.add(DESKTOP_QUERY, () => {
        // Без списка (пустые данные) карточка не смонтирована: анимацию не строим.
        if (!list) return;
        gsap.set(fly, { autoAlpha: 0, scale: 0.9 });
        gsap.set(card, { autoAlpha: 0, scaleY: 0.7, transformOrigin: "top center" });
        gsap.set(badge, { autoAlpha: 0, scale: 0.8 });

        const lengths = shield.paths.map((p) => p.getTotalLength());
        shield.paths.forEach((p, i) => {
          p.style.strokeDasharray = String(lengths[i]);
          p.style.strokeDashoffset = String(lengths[i]);
        });

        const tl = gsap.timeline({
          defaults: { ease: "none" },
          scrollTrigger: {
            trigger: shell.section,
            start: "top top",
            end: "+=170%",
            pin: true,
            scrub: 0.6,
            anticipatePin: 1,
            invalidateOnRefresh: true,
          },
        });

        tl.fromTo(winWrap, { y: 28, opacity: 0 }, { y: 0, opacity: 1, duration: 0.12, ease: "power1.out" }, 0);

        tl.to(fly, { autoAlpha: 1, scale: 1, duration: 0.06 }, 0.14);
        tl.to(fly, {
          x: () => {
            const from = layoutCenter(fly, wrap);
            const to = layoutCenter(card, wrap);
            return to.x - from.x;
          },
          y: () => {
            const from = layoutCenter(fly, wrap);
            const to = layoutCenter(card, wrap);
            return to.y - from.y;
          },
          scale: 0.55,
          duration: 0.3,
          ease: "power1.inOut",
        }, 0.2);

        tl.to(fly, { autoAlpha: 0, duration: 0.05 }, 0.5);
        tl.to(card, { autoAlpha: 1, scaleY: 1, duration: 0.14, ease: "back.out(1.4)" }, 0.52);

        tl.to(shield.paths[0], { strokeDashoffset: 0, duration: 0.16 }, 0.64);
        tl.to(shield.paths[1], { strokeDashoffset: 0, duration: 0.1 }, 0.78);

        tl.to(badge, { autoAlpha: 1, scale: 1, duration: 0.1, ease: "back.out(1.6)" }, 0.86);
        tl.to(card, { "--glow": "1", duration: 0.1 }, 0.9);
      });
    },
    cleanup() {
      app.cleanup();
    },
  };
}
