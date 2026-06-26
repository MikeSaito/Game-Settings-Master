import type { FeatureStrings } from "../../i18n/types";
import { createAppShot } from "../shared/AppShot";

export function createFeatureSection(feature: FeatureStrings): HTMLElement {
  const article = document.createElement("article");
  article.className = `feature${feature.reverse ? " feature--reverse" : ""}`;
  article.id = feature.id;
  article.innerHTML = `
    <div class="feature__content">
      <span class="feature__step">${feature.step}</span>
      <h2 class="feature__title">${feature.title}</h2>
      <p class="feature__text">${feature.text}</p>
    </div>
    <div class="feature__visual"></div>
  `;
  article.querySelector(".feature__visual")?.appendChild(
    createAppShot({ src: feature.shot, alt: feature.title, variant: "card" }),
  );

  const observer = new IntersectionObserver(
    ([entry]) => {
      if (entry?.isIntersecting) {
        article.classList.add("is-visible");
        observer.disconnect();
      }
    },
    { threshold: 0.15, rootMargin: "0px 0px -6% 0px" },
  );
  observer.observe(article);

  return article;
}
