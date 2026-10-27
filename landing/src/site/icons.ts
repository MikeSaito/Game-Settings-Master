const paths: Record<string, string> = {
  arrow: '<path d="M5 12h14m-6-6 6 6-6 6"/>',
  northeast: '<path d="M7 17 17 7M7 7h10v10"/>',
  download: '<path d="M12 3v12m-5-5 5 5 5-5M4 16v4h16v-4"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5"/>',
  moon: '<path d="M20 14a8 8 0 0 1-10-10A8.5 8.5 0 1 0 20 14Z"/>',
  sliders:
    '<path d="M4 7h9m4 0h3M4 17h3m4 0h9"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="17" r="2"/>',
  preview: '<path d="M4 3h11l5 5v13H4zM14 3v6h6M8 13h8m-8 4h5"/>',
  restore: '<path d="M3 10a9 9 0 1 1 1 8M3 4v6h6m3-4v6l4 2"/>',
  diagnostics: '<path d="M3 12h4l3-7 4 14 3-7h4"/>',
  code: '<path d="m8 6-6 6 6 6m8-12 6 6-6 6M14 3l-4 18"/>',
  preset:
    '<rect x="4" y="3" width="16" height="18" rx="3"/><path d="M8 8h8m-8 4h4m-4 4h6"/>',
  gpu: '<rect x="5" y="5" width="14" height="14" rx="3"/><path d="M9 1v4m6-4v4M9 19v4m6-4v4M1 9h4m-4 6h4m14-6h4m-4 6h4"/><rect x="9" y="9" width="6" height="6" rx="1"/>',
  input:
    '<rect x="2" y="5" width="20" height="14" rx="3"/><path d="M6 9h.01M10 9h.01M14 9h.01M18 9h.01M6 13h.01M10 13h.01M14 13h.01M18 13h.01M8 16h8"/>',
  game: '<path d="M7 6h10c3 0 5 11 3 12-2 1-4-3-5-3H9c-1 0-3 4-5 3C2 17 4 6 7 6Z"/><path d="M7 9v4m-2-2h4m6-1h.01m3 2h.01"/>',
  chevron: '<path d="m8 10 4 4 4-4"/>',
  github:
    '<path d="M9 20c-5 1-5-2-7-2m14 4v-4c0-1-.3-2-1-2 4-.5 6-2 6-5 0-1.5-.5-2.5-1.5-3.5C20 6 20 5 19.5 4c-2 0-3 1-4 1-2-.5-4-.5-6 0-1-1-2-1-4-1C5 5 5 6 5.5 7.5 4.5 8.5 4 9.5 4 11c0 3 2 4.5 6 5-.7.3-1 1-1 2v4"/>',
};
export function icon(name: string, className = ""): string {
  return `<svg class="icon ${className}" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths[name] ?? paths.arrow}</svg>`;
}
