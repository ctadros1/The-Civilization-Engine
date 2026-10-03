// The Civilization Engine's web observer (M0): connects to civ-host, shows the world map and its
// panels, and sends the player's commands. The kernel owns truth; this page only shows it and
// asks for changes.

import "./styles.css";

import { formatDistance } from "./format.js";
import { MapView, type PointerInfo } from "./map/view.js";
import { HostClient } from "./net/client.js";
import * as M from "./net/messages.js";
import { Store, initialState, mergeEvents } from "./state.js";
import { bindUi } from "./ui.js";

function socketUrl(): string {
  const host = new URLSearchParams(location.search).get("host");
  if (host) return `ws://${host}/ws`;
  return `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws`;
}

function byId(id: string): HTMLElement {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing #${id}`);
  return node;
}

const store = new Store(initialState());
const map = new MapView();
/** `epoch:worldId` of the world on the map. */
let shownWorld = "";

const client = new HostClient(socketUrl(), {
  status: (connection) => store.update({ connection }),
  welcome: (welcome) => {
    // A new connection may be to a restarted host: redraw from scratch.
    shownWorld = "";
    store.update({ welcome, events: [] });
  },
  snapshot: (snapshot, epoch) => {
    store.update({ snapshot, epoch });
    syncMap();
  },
  events: (items) => store.update({ events: mergeEvents(store.state.events, items) }),
});

function syncMap(): void {
  if (!store.state.mapAvailable) return;
  const world = store.state.snapshot?.world ?? null;
  const key = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (key === shownWorld) return;
  shownWorld = key;
  if (world) void map.show(world, client);
  else map.clear();
}

async function command(payload: Uint8Array): Promise<void> {
  await client.command(payload);
}

map.onStatus = (status) => store.update({ map: status });

const readout = byId("readout");
const IDLE_READOUT = "Point at the map for coordinates and elevation";
readout.textContent = IDLE_READOUT;
readout.classList.add("idle");
map.onPointer = (info: PointerInfo | null) => {
  readout.classList.toggle("idle", !info);
  if (!info) {
    readout.textContent = IDLE_READOUT;
    return;
  }
  const height = info.elevationM === null ? "" : ` · ${Math.round(info.elevationM)} m`;
  readout.textContent =
    `${formatDistance(info.xM)} E, ${formatDistance(info.yM)} S · cell ${info.cellX}, ${info.cellY}` +
    `${height} · ${info.water ?? ""}`;
};

const scaleBar = byId("scalebar-bar");
const scaleText = byId("scalebar-text");
map.onCamera = () => {
  const pxPerM = map.scale;
  if (!(pxPerM > 0)) return;
  let metres = 1;
  for (const step of [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10_000, 20_000]) {
    metres = step;
    if (step * pxPerM >= 70) break;
  }
  scaleBar.style.width = `${Math.round(metres * pxPerM)}px`;
  scaleText.textContent = formatDistance(metres);
};

byId("zoom-in").addEventListener("click", () => map.zoomBy(1.5));
byId("zoom-out").addEventListener("click", () => map.zoomBy(1 / 1.5));
byId("zoom-fit").addEventListener("click", () => map.fit());
byId("show-rivers").addEventListener("change", (event) => {
  map.setRiversVisible((event.target as HTMLInputElement).checked);
});

bindUi(store, {
  newWorld: (args) => command(M.newWorld(args)),
  save: (label) => command(M.saveWorld(label)),
  listSaves: () => client.saves(),
  load: (file) => command(M.loadWorld(file)),
  setClock: (paused, speed) => command(M.setClock(paused, speed)),
  cancelTask: () => command(M.cancelTask()),
  recover: (accept) => command(M.recoverWorld(accept)),
  retryMap: () => {
    shownWorld = "";
    syncMap();
  },
});

/** Test and debugging hooks (mirrors Genesis's window.__OBS__). Plain data only. */
const hooks = {
  state: () => {
    const s = store.state;
    const world = s.snapshot?.world;
    return {
      connection: s.connection.state,
      welcomed: !!s.welcome,
      epoch: s.epoch,
      world: world ? { ...world, seed: world.seed.toString() } : null,
      clock: s.snapshot?.clock ?? null,
      task: s.snapshot?.task ?? null,
      recovery: s.snapshot?.recovery ?? null,
      lastError: s.snapshot?.lastError ?? null,
      events: s.events.map((e) => ({ kind: e.kind, text: e.text })),
    };
  },
  map: () => map.debugState(),
  pointerAt: (x: number, y: number) => map.pointerInfo(x, y),
  panBy: (dx: number, dy: number) => map.panBy(dx, dy),
  zoomBy: (factor: number) => map.zoomBy(factor),
  fit: () => map.fit(),
};

declare global {
  interface Window {
    __TCE__: typeof hooks;
  }
}
window.__TCE__ = hooks;

async function start(): Promise<void> {
  try {
    await map.mount(byId("map"));
  } catch (e) {
    store.update({
      mapAvailable: false,
      map: {
        state: "error",
        message: `WebGL is not available (${e instanceof Error ? e.message : String(e)}). The panels still work.`,
      },
    });
  }
  client.start();
}

void start();
