import type { HighlightStrings } from "../../i18n/types";

export function createGpuSection(gpu: HighlightStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "gpu-section section page-wrap reveal-stagger";
  section.id = "gpu";
  section.innerHTML = `
    <div class="gpu-panel">
      <div>
        <span class="section-heading__eyebrow">${gpu.eyebrow}</span>
        <h2>${gpu.title}</h2>
        <p>${gpu.text}</p>
      </div>
      <ul class="gpu-list">
        ${gpu.bullets.map((bullet) => `<li>${bullet}</li>`).join("")}
      </ul>
    </div>
  `;
  return section;
}
