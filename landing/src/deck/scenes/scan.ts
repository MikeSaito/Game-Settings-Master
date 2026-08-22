import type { LocaleStrings } from "../../i18n/types";
import { assetPath } from "../../lib/site";
import { buildAppWindow } from "../../mock/appWindow";
import { gsap, DESKTOP_QUERY, type Scene } from "../scroll";
import { sceneShell } from "./shell";

export function buildScanScene(t: LocaleStrings): Scene {
  const shell = sceneShell("scan", t.story.scan);

  const wrap = document.createElement("div");
  wrap.className = "scan";

  const icon = document.createElement("div");
  icon.className = "scan__icon";
  const iconImg = document.createElement("img");
  iconImg.src = assetPath("logo.svg");
  iconImg.alt = "";
  iconImg.width = 56;
  iconImg.height = 56;
  const iconLabel = document.createElement("span");
  iconLabel.textContent = t.siteName;
  icon.append(iconImg, iconLabel);

  const winWrap = document.createElement("div");
  winWrap.className = "scan__win";
  const app = buildAppWindow(t, { initial: "library" });
  winWrap.append(app.el);

  const beam = document.createElement("div");
  beam.className = "scan__beam";
  beam.setAttribute("aria-hidden", "true");

  const counter = document.createElement("p");
  counter.className = "scan__counter";
  counter.setAttribute("aria-hidden", "true");
  counter.textContent = t.story.scan.counter(0);

  const status = document.createElement("p");
  status.className = "scan__status";
  status.setAttribute("aria-hidden", "true");
  status.textContent = t.story.scan.scanning;

  winWrap.append(beam, counter, status);
  wrap.append(icon, winWrap);
  shell.stage.append(wrap);

  const cards = Array.from(app.el.querySelectorAll<HTMLElement>(".mock__card"));

  return {
    el: shell.section,
    init(mm) {
      mm.add(DESKTOP_QUERY, () => {
        gsap.set(winWrap, { opacity: 0, scale: 0.3 });
        gsap.set(cards, { opacity: 0, y: 16, clipPath: "inset(0 0 100% 0)" });
        gsap.set([counter, status], { autoAlpha: 0 });
        gsap.set(beam, { y: -90, opacity: 0 });

        const found = { n: 0 };

        const tl = gsap.timeline({
          defaults: { ease: "none" },
          scrollTrigger: {
            trigger: shell.section,
            start: "top top",
            end: "+=200%",
            pin: true,
            scrub: 0.6,
            anticipatePin: 1,
            invalidateOnRefresh: true,
          },
        });

        tl.to(icon, { scale: 0.7, opacity: 0, duration: 0.1, ease: "power1.in" }, 0);
        tl.to(winWrap, { opacity: 1, scale: 1, duration: 0.2, ease: "power2.out" }, 0.04);

        tl.to(status, { autoAlpha: 1, duration: 0.05 }, 0.24);
        tl.to(counter, { autoAlpha: 1, duration: 0.05 }, 0.3);

        tl.fromTo(beam, { y: -90 }, { y: () => winWrap.offsetHeight + 20, duration: 0.42 }, 0.3);
        tl.to(beam, { opacity: 1, duration: 0.04 }, 0.3);
        tl.to(beam, { opacity: 0, duration: 0.05 }, 0.7);

        tl.to(cards, {
          opacity: 1,
          y: 0,
          clipPath: "inset(0 0 0% 0)",
          duration: 0.26,
          stagger: 0.07,
          ease: "power1.out",
        }, 0.34);

        tl.to(found, {
          n: cards.length,
          duration: 0.34,
          onUpdate: () => {
            counter.textContent = t.story.scan.counter(Math.round(found.n));
          },
        }, 0.34);

        tl.to(status, { autoAlpha: 0, duration: 0.05 }, 0.74);
        tl.to(winWrap, { "--glow": "1", duration: 0.12 }, 0.82);
        tl.to(winWrap, { "--glow": "0.45", duration: 0.1 }, 0.94);
      });
    },
    cleanup() {
      app.cleanup();
    },
  };
}
