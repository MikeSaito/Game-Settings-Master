let x = window.innerWidth * 0.5;
let y = window.innerHeight * 0.5;
let vx = 0;
let vy = 0;
let active = false;

export function initMouseState(): () => void {
  let lastX = x;
  let lastY = y;

  const onMove = (event: PointerEvent) => {
    vx = event.clientX - lastX;
    vy = event.clientY - lastY;
    lastX = event.clientX;
    lastY = event.clientY;
    x = event.clientX;
    y = event.clientY;
    active = true;
  };

  const onLeave = () => {
    active = false;
    vx = 0;
    vy = 0;
  };

  window.addEventListener("pointermove", onMove, { passive: true });
  document.documentElement.addEventListener("mouseleave", onLeave);
  return () => {
    window.removeEventListener("pointermove", onMove);
    document.documentElement.removeEventListener("mouseleave", onLeave);
  };
}

export function getMouseState() {
  return { x, y, vx, vy, active };
}
