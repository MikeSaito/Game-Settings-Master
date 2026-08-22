import type { LocaleStrings } from "../../i18n/types";
import { buildAppWindow } from "../../mock/appWindow";
import { makeActions } from "../../ui/cta";
import { gsap, DESKTOP_QUERY, type Scene } from "../scroll";
import { sceneShell } from "./shell";

export function buildFinalScene(t: LocaleStrings): Scene {
  const shell = sceneShell("final", t.story.final);
  shell.section.classList.add("scene--center");

  const wrap = document.createElement("div");
  wrap.className = "final";

  const monitor = document.createElement("div");
  monitor.className = "final__monitor";
  const screen = document.createElement("div");
  screen.className = "final__screen";
  const app = buildAppWindow(t, { initial: "library" });
  screen.append(app.el);
  const stand = document.createElement("div");
  stand.className = "final__stand";
  stand.setAttribute("aria-hidden", "true");
  monitor.append(screen, stand);

  const cta = document.createElement("div");
  cta.className = "final__cta";
  cta.append(makeActions(t));

  const badges = document.createElement("ul");
  badges.className = "final__badges";
  for (const text of t.story.final.badges) {
    const li = document.createElement("li");
    li.textContent = text;
    badges.append(li);
  }

  const tagline = document.createElement("p");
  tagline.className = "final__tagline";
  tagline.textContent = t.story.final.tagline;

  wrap.append(monitor, cta, badges, tagline);
  shell.stage.append(wrap);

  return {
    el: shell.section,
    init(mm) {
      mm.add(DESKTOP_QUERY, () => {
        gsap.set(shell.copy, { opacity: 0, y: 26 });
        gsap.set(monitor, { opacity: 0, y: 60, scale: 0.94 });
        gsap.set(screen, { rotateX: 16, scale: 1.08, transformPerspective: 900 });
        gsap.set(cta, { opacity: 0, scale: 0.8 });
        gsap.set(badges.children, { opacity: 0, y: 14 });
        gsap.set(tagline, { opacity: 0 });

        const tl = gsap.timeline({
          defaults: { ease: "none" },
          scrollTrigger: {
            trigger: shell.section,
            start: "top top",
            end: "+=170%",
            pin: true,
            scrub: 0.6,
            anticipatePin: 1,
          },
        });

        tl.to(monitor, { opacity: 1, y: 0, scale: 1, duration: 0.22, ease: "power1.out" }, 0);
        tl.to(screen, { rotateX: 0, scale: 1, duration: 0.26, ease: "power1.out" }, 0.02);
        tl.to(shell.copy, { opacity: 1, y: 0, duration: 0.16, ease: "power1.out" }, 0.14);
        tl.to(cta, { opacity: 1, scale: 1, duration: 0.16, ease: "back.out(1.5)" }, 0.36);
        tl.to(badges.children, { opacity: 1, y: 0, duration: 0.12, stagger: 0.05 }, 0.56);
        tl.to(tagline, { opacity: 1, duration: 0.14 }, 0.74);
      });
    },
    cleanup() {
      app.cleanup();
    },
  };
}
