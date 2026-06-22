import type { FeatureStrings } from "../../i18n/types";

export function createFeatureSection(feature: FeatureStrings): HTMLElement {
  const article = document.createElement("article");
  article.className = `feature${feature.reverse ? " feature--reverse" : ""}`;
  article.id = feature.id;
  article.innerHTML = `
    <div class="feature__content">
      <div class="feature__step">${feature.step}</div>
      <h2 class="feature__title">${feature.title}</h2>
      <p class="feature__text">${feature.text}</p>
    </div>
    <div class="feature__illus" aria-hidden="true">${feature.illustration}</div>
  `;

  const observer = new IntersectionObserver(
    ([entry]) => {
      if (entry?.isIntersecting) {
        article.classList.add("is-visible");
        observer.disconnect();
      }
    },
    { threshold: 0.2, rootMargin: "0px 0px -10% 0px" },
  );
  observer.observe(article);

  return article;
}
