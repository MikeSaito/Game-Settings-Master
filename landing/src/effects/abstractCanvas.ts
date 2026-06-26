import { getMouseState } from "./mouseState";

interface Vec3 {
  x: number;
  y: number;
  z: number;
}

interface Face {
  i: number;
  j: number;
  k: number;
}

interface Proj {
  x: number;
  y: number;
  z: number;
  f: number;
}

interface Cluster {
  origin: Vec3;
  scale: number;
  rays: number;
  spin: number;
}

interface ClusterMesh {
  cluster: Cluster;
  verts: Vec3[];
  faces: Face[];
}

interface PlexusNode {
  pos: Vec3;
  drift: Vec3;
  phase: number;
}

interface Particle {
  pos: Vec3;
  size: number;
  phase: number;
}

const LIGHT = normalize({ x: 0.3, y: -0.7, z: 0.64 });
const AMBER_DARK = { r: 120, g: 74, b: 14 };
const AMBER_LIGHT = { r: 220, g: 162, b: 58 };

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function seededRandom(seed: number): () => number {
  let state = seed % 2147483647;
  if (state <= 0) state += 2147483646;
  return () => {
    state = (state * 16807) % 2147483647;
    return (state - 1) / 2147483646;
  };
}

function normalize(v: Vec3): Vec3 {
  const len = Math.hypot(v.x, v.y, v.z) || 1;
  return { x: v.x / len, y: v.y / len, z: v.z / len };
}

function cross(a: Vec3, b: Vec3): Vec3 {
  return {
    x: a.y * b.z - a.z * b.y,
    y: a.z * b.x - a.x * b.z,
    z: a.x * b.y - a.y * b.x,
  };
}

function dot(a: Vec3, b: Vec3): number {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

function sub(a: Vec3, b: Vec3): Vec3 {
  return { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z };
}

function applyScrollParallax(v: Vec3, scroll: number): Vec3 {
  const depth = 0.25 + Math.min(Math.max(v.z, 0) / 160, 1) * 0.75;
  const driftY = -scroll * 0.028 * depth;
  const driftZ = -scroll * 0.01 * depth;

  return {
    x: v.x,
    y: v.y + driftY,
    z: v.z - driftZ,
  };
}

function transformVertex(
  v: Vec3,
  cluster: Cluster,
  time: number,
  pivot: Vec3,
  scroll: number,
): Vec3 {
  let p = rotateY(v, time * cluster.spin * 0.004, cluster.origin);
  p = rotateY(p, time * 0.005, pivot);
  p = applyScrollParallax(p, scroll);
  return p;
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

function rotateY(v: Vec3, angle: number, origin: Vec3): Vec3 {
  const x = v.x - origin.x;
  const z = v.z - origin.z;
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return {
    x: origin.x + x * c - z * s,
    y: v.y,
    z: origin.z + x * s + z * c,
  };
}

function faceNormal(a: Vec3, b: Vec3, c: Vec3): Vec3 {
  return normalize(cross(sub(b, a), sub(c, a)));
}

function shadeFace(a: Vec3, b: Vec3, c: Vec3): number {
  const n = faceNormal(a, b, c);
  return Math.min(1, Math.max(0.06, 0.12 + dot(n, LIGHT) * 0.86));
}

function mixAmber(t: number): string {
  const x = Math.min(1, Math.max(0, t));
  const r = Math.round(lerp(AMBER_DARK.r, AMBER_LIGHT.r, x));
  const g = Math.round(lerp(AMBER_DARK.g, AMBER_LIGHT.g, x));
  const b = Math.round(lerp(AMBER_DARK.b, AMBER_LIGHT.b, x));
  return `rgb(${r}, ${g}, ${b})`;
}

function buildBurstCluster(
  origin: Vec3,
  clusterScale: number,
  rays: number,
): { verts: Vec3[]; faces: Face[] } {
  const tips: Vec3[] = [];
  const golden = Math.PI * (3 - Math.sqrt(5));

  for (let i = 0; i < rays; i += 1) {
    const t = (i + 0.5) / rays;
    const phi = Math.acos(1 - 2 * t);
    const theta = golden * i;
    const len = clusterScale * (0.88 + (i % 3) * 0.04);
    tips.push({
      x: origin.x + Math.sin(phi) * Math.cos(theta) * len,
      y: origin.y + Math.sin(phi) * Math.sin(theta) * len * 0.9,
      z: origin.z + Math.cos(phi) * len * 0.72,
    });
  }

  const core = { x: origin.x, y: origin.y, z: origin.z + clusterScale * 0.12 };
  const verts = [core, ...tips];
  const faces: Face[] = [];

  for (let i = 0; i < rays; i += 1) {
    faces.push({ i: 0, j: 1 + i, k: 1 + ((i + 1) % rays) });
  }

  return { verts, faces };
}

function subdivideClean(verts: Vec3[], faces: Face[]): { verts: Vec3[]; faces: Face[] } {
  const v = [...verts];
  const next: Face[] = [];

  for (const face of faces) {
    const a = v[face.i];
    const b = v[face.j];
    const c = v[face.k];
    const ab = { x: (a.x + b.x) * 0.5, y: (a.y + b.y) * 0.5, z: (a.z + b.z) * 0.5 };
    const bc = { x: (b.x + c.x) * 0.5, y: (b.y + c.y) * 0.5, z: (b.z + c.z) * 0.5 };
    const ca = { x: (c.x + a.x) * 0.5, y: (c.y + a.y) * 0.5, z: (c.z + a.z) * 0.5 };
    const iab = v.length;
    v.push(ab);
    const ibc = v.length;
    v.push(bc);
    const ica = v.length;
    v.push(ca);
    next.push(
      { i: face.i, j: iab, k: ica },
      { i: iab, j: face.j, k: ibc },
      { i: ica, j: ibc, k: face.k },
      { i: iab, j: ibc, k: ica },
    );
  }

  return { verts: v, faces: next };
}

function buildClusters(width: number, height: number): Cluster[] {
  const s = Math.max(width, height);
  return [
    { origin: { x: width * 0.56, y: height * 0.48, z: 95 }, scale: s * 0.24, rays: 20, spin: 0.04 },
    { origin: { x: width * 0.78, y: height * 0.28, z: 48 }, scale: s * 0.11, rays: 13, spin: -0.05 },
    { origin: { x: width * 0.22, y: height * 0.62, z: 42 }, scale: s * 0.1, rays: 12, spin: 0.05 },
    { origin: { x: width * 0.74, y: height * 0.72, z: 34 }, scale: s * 0.085, rays: 11, spin: -0.04 },
    { origin: { x: width * 0.28, y: height * 0.24, z: 28 }, scale: s * 0.078, rays: 10, spin: 0.04 },
    { origin: { x: width * 0.86, y: height * 0.46, z: 24 }, scale: s * 0.07, rays: 9, spin: -0.03 },
  ];
}

function buildClusterMeshes(clusters: Cluster[]): ClusterMesh[] {
  return clusters.map((cluster, idx) => {
    const burst = buildBurstCluster(cluster.origin, cluster.scale, cluster.rays);
    const mesh = idx === 0 ? subdivideClean(burst.verts, burst.faces) : burst;
    return { cluster, verts: mesh.verts, faces: mesh.faces };
  });
}

function buildPlexusNodes(
  meshVerts: Vec3[],
  width: number,
  height: number,
  rand: () => number,
): PlexusNode[] {
  const cx = width * 0.52;
  const cy = height * 0.5;
  const spread = Math.max(width, height) * 0.44;
  const nodes: PlexusNode[] = [];

  for (let i = 0; i < meshVerts.length; i += 2) {
    const p = meshVerts[i];
    nodes.push({
      pos: p,
      drift: { x: (rand() - 0.5) * 0.4, y: (rand() - 0.5) * 0.4, z: (rand() - 0.5) * 0.3 },
      phase: rand() * Math.PI * 2,
    });
  }

  for (let i = 0; i < 48; i += 1) {
    const u = rand();
    const v = rand();
    nodes.push({
      pos: {
        x: cx + (u - 0.5) * spread * 1.85,
        y: cy + (v - 0.5) * spread * 1.35,
        z: rand() * 130 - 10,
      },
      drift: { x: (rand() - 0.5) * 0.15, y: (rand() - 0.5) * 0.15, z: (rand() - 0.5) * 0.1 },
      phase: rand() * Math.PI * 2,
    });
  }

  return nodes;
}

function buildParticles(width: number, height: number, rand: () => number): Particle[] {
  return Array.from({ length: 36 }, () => ({
    pos: {
      x: rand() * width,
      y: rand() * height,
      z: rand() * 140,
    },
    size: 0.8 + rand() * 2.2,
    phase: rand() * Math.PI * 2,
  }));
}

function project(
  v: Vec3,
  width: number,
  height: number,
  parallaxX: number,
  parallaxY: number,
): Proj {
  const f = 740 / (740 + v.z);
  return {
    x: (v.x - width * 0.5) * f + width * 0.5 + parallaxX * v.z * 0.00004,
    y: (v.y - height * 0.5) * f + height * 0.5 + parallaxY * v.z * 0.00003,
    z: v.z,
    f,
  };
}

function transformPlexusNode(
  node: PlexusNode,
  time: number,
  pivot: Vec3,
  scroll: number,
): Vec3 {
  let p = animateNode(node, time);
  p = rotateY(p, time * 0.005, pivot);
  p = applyScrollParallax(p, scroll);
  return p;
}

function avgZ(a: Proj, b: Proj, c: Proj): number {
  return (a.z + b.z + c.z) / 3;
}

function dist3(a: Vec3, b: Vec3): number {
  return Math.hypot(a.x - b.x, a.y - b.y, a.z - b.z);
}

function animateNode(node: PlexusNode, time: number): Vec3 {
  const wobble = Math.sin(time * 0.35 + node.phase) * 2;
  return {
    x: node.pos.x + node.drift.x * wobble,
    y: node.pos.y + node.drift.y * wobble,
    z: node.pos.z + node.drift.z * wobble,
  };
}

function drawStudio(ctx: CanvasRenderingContext2D, width: number, height: number) {
  const bg = ctx.createLinearGradient(0, 0, 0, height);
  bg.addColorStop(0, "#faf8f4");
  bg.addColorStop(0.55, "#f5f2ea");
  bg.addColorStop(1, "#e8e2d6");
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, width, height);

  const floor = ctx.createRadialGradient(
    width * 0.5,
    height * 0.92,
    width * 0.05,
    width * 0.5,
    height * 0.55,
    width * 0.72,
  );
  floor.addColorStop(0, "rgba(184, 122, 20, 0.04)");
  floor.addColorStop(1, "rgba(245, 242, 234, 0)");
  ctx.fillStyle = floor;
  ctx.fillRect(0, 0, width, height);
}

export function initAbstractCanvas(): (() => void) | undefined {
  if (prefersReducedMotion()) return undefined;

  const canvas = document.createElement("canvas");
  canvas.className = "fx-canvas";
  canvas.setAttribute("aria-hidden", "true");
  document.body.prepend(canvas);

  const ctx = canvas.getContext("2d");
  if (!ctx) return undefined;

  let width = 0;
  let height = 0;
  let dpr = 1;
  let raf = 0;
  let start = performance.now();
  let scrollY = 0;
  let clusterMeshes: ClusterMesh[] = [];
  let plexus: PlexusNode[] = [];
  let particles: Particle[] = [];
  let scenePivot = { x: 0, y: 0, z: 60 };

  const resize = () => {
    dpr = Math.min(window.devicePixelRatio || 1, 2);
    width = window.innerWidth;
    height = window.innerHeight;
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const rand = seededRandom(width * 23 + height * 17);
    const clusters = buildClusters(width, height);
    clusterMeshes = buildClusterMeshes(clusters);
    const allVerts = clusterMeshes.flatMap((m) => m.verts);
    plexus = buildPlexusNodes(allVerts, width, height, rand);
    particles = buildParticles(width, height, rand);
    scenePivot = { x: width * 0.54, y: height * 0.46, z: 60 };
  };

  const syncScroll = () => {
    scrollY = window.scrollY;
  };

  const draw = (now: number) => {
    const time = (now - start) * 0.001;
    const mouse = getMouseState();
    const px = mouse.active ? mouse.x - width * 0.5 : 0;
    const py = mouse.active ? mouse.y - height * 0.5 : 0;

    ctx.clearRect(0, 0, width, height);
    drawStudio(ctx, width, height);

    const drawItems: Array<{
      face: Face;
      world: [Vec3, Vec3, Vec3];
      proj: [Proj, Proj, Proj];
    }> = [];

    for (const mesh of clusterMeshes) {
      const worldVerts = mesh.verts.map((v) =>
        transformVertex(v, mesh.cluster, time, scenePivot, scrollY),
      );
      const projectedVerts = worldVerts.map((v) => project(v, width, height, px, py));

      for (const face of mesh.faces) {
        drawItems.push({
          face,
          world: [worldVerts[face.i], worldVerts[face.j], worldVerts[face.k]],
          proj: [projectedVerts[face.i], projectedVerts[face.j], projectedVerts[face.k]],
        });
      }
    }

    drawItems.sort((a, b) => avgZ(a.proj[0], a.proj[1], a.proj[2]) - avgZ(b.proj[0], b.proj[1], b.proj[2]));

    for (const item of drawItems) {
      const [va, vb, vc] = item.world;
      const [a, b, c] = item.proj;
      const shade = shadeFace(va, vb, vc);

      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.lineTo(c.x, c.y);
      ctx.closePath();
      ctx.fillStyle = mixAmber(shade);
      ctx.globalAlpha = 0.62 + shade * 0.36;
      ctx.fill();
      ctx.globalAlpha = 1;

      ctx.strokeStyle = `rgba(255, 255, 255, ${0.15 + shade * 0.35})`;
      ctx.lineWidth = 0.85;
      ctx.stroke();
    }

    const liveNodes = plexus.map((n) => transformPlexusNode(n, time, scenePivot, scrollY));
    const projectedNodes = liveNodes.map((v) => project(v, width, height, px, py));
    const linkDist = Math.max(width, height) * 0.24;

    const links: Array<{ a: number; b: number; z: number }> = [];
    for (let i = 0; i < liveNodes.length; i += 1) {
      for (let j = i + 1; j < liveNodes.length; j += 1) {
        const d = dist3(liveNodes[i], liveNodes[j]);
        if (d > linkDist) continue;
        links.push({
          a: i,
          b: j,
          z: (projectedNodes[i].z + projectedNodes[j].z) * 0.5,
        });
      }
    }
    links.sort((a, b) => a.z - b.z);

    for (const link of links) {
      const pa = projectedNodes[link.a];
      const pb = projectedNodes[link.b];
      const d = dist3(liveNodes[link.a], liveNodes[link.b]);
      const t = 1 - d / linkDist;

      ctx.beginPath();
      ctx.moveTo(pa.x, pa.y);
      ctx.lineTo(pb.x, pb.y);
      ctx.strokeStyle = `rgba(184, 122, 20, ${0.06 + t * 0.22})`;
      ctx.lineWidth = 0.65 + t * 0.35;
      ctx.stroke();
    }

    for (let i = 0; i < liveNodes.length - 2; i += 3) {
      const pa = projectedNodes[i];
      const pb = projectedNodes[i + 1];
      const pc = projectedNodes[i + 2];
      const z = (pa.z + pb.z + pc.z) / 3;

      ctx.beginPath();
      ctx.moveTo(pa.x, pa.y);
      ctx.lineTo(pb.x, pb.y);
      ctx.lineTo(pc.x, pc.y);
      ctx.closePath();
      ctx.fillStyle = `rgba(184, 122, 20, ${0.02 + (1 - z / 140) * 0.04})`;
      ctx.fill();
    }

    for (const p of projectedNodes) {
      ctx.beginPath();
      ctx.arc(p.x, p.y, 2 * p.f, 0, Math.PI * 2);
      ctx.fillStyle = "rgba(184, 122, 20, 0.75)";
      ctx.fill();
      ctx.strokeStyle = "rgba(255, 255, 255, 0.35)";
      ctx.lineWidth = 0.5;
      ctx.stroke();
    }

    for (const particle of particles) {
      let pos: Vec3 = {
        x: particle.pos.x,
        y: particle.pos.y + Math.sin(time * 0.5 + particle.phase) * 8,
        z: particle.pos.z,
      };
      pos = rotateY(pos, time * 0.012, scenePivot);
      pos = applyScrollParallax(pos, scrollY);
      const p = project(pos, width, height, px, py);
      const alpha = 0.15 + (1 - p.z / 160) * 0.45;

      ctx.beginPath();
      ctx.arc(p.x, p.y, particle.size * p.f, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(255, 255, 255, ${alpha})`;
      ctx.fill();
    }

    const haze = ctx.createRadialGradient(
      width * 0.26,
      height * 0.34,
      width * 0.04,
      width * 0.26,
      height * 0.34,
      width * 0.38,
    );
    haze.addColorStop(0, "rgba(245, 242, 234, 0.3)");
    haze.addColorStop(1, "rgba(245, 242, 234, 0)");
    ctx.fillStyle = haze;
    ctx.fillRect(0, 0, width, height);

    const vignette = ctx.createRadialGradient(
      width * 0.5,
      height * 0.48,
      Math.min(width, height) * 0.22,
      width * 0.5,
      height * 0.48,
      Math.max(width, height) * 0.82,
    );
    vignette.addColorStop(0, "rgba(245, 242, 234, 0)");
    vignette.addColorStop(1, "rgba(50, 40, 28, 0.14)");
    ctx.fillStyle = vignette;
    ctx.fillRect(0, 0, width, height);

    raf = requestAnimationFrame(draw);
  };

  const onScroll = () => {
    syncScroll();
  };

  resize();
  syncScroll();
  window.addEventListener("resize", resize, { passive: true });
  window.addEventListener("scroll", onScroll, { passive: true });
  raf = requestAnimationFrame(draw);

  return () => {
    cancelAnimationFrame(raf);
    window.removeEventListener("resize", resize);
    window.removeEventListener("scroll", onScroll);
    canvas.remove();
  };
}
