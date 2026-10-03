// The Civilization Engine's web observer: connects to civ-host, shows the world map, its people
// and its panels, and sends the player's commands. The kernel owns truth; this page only shows it
// and asks for changes.

import "./styles.css";

import { formatDistance } from "./format.js";
import { MapView, type PointerInfo } from "./map/view.js";
import { HostClient, HostError } from "./net/client.js";
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

/** Decision receipts the inspector asks for. */
const DECISIONS_SHOWN = 8;
/** Least real time between refreshes of the inspected person, milliseconds. */
const PERSON_REFRESH_MS = 1000;

const client = new HostClient(socketUrl(), {
  status: (connection) => store.update({ connection }),
  welcome: (welcome) => {
    // A new connection may be to a restarted host: redraw from scratch.
    shownWorld = "";
    map.setActivities(welcome.activities);
    store.update({ welcome, events: [], chronicle: [], selected: null });
    map.setSelected(null);
  },
  snapshot: (snapshot, epoch) => {
    store.update({ snapshot, epoch });
    syncMap();
    map.setPeople(snapshot.people, snapshot.settlements, snapshot.clock);
    void syncChronicle();
    void syncFields();
    void syncBuildings();
    void refreshPerson(false);
  },
  events: (items) => store.update({ events: mergeEvents(store.state.events, items) }),
});

/** The world the chronicle and selection belong to. */
let historyWorld = "";
let chronicleBusy = false;

/** Fetches chronicle entries newer than the ones held. */
async function syncChronicle(): Promise<void> {
  const world = store.state.snapshot?.world;
  const key = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (key !== historyWorld) {
    historyWorld = key;
    store.update({ chronicle: [], selected: null });
    map.setSelected(null);
  }
  const head = store.state.snapshot?.chronicleHead ?? 0;
  const have = store.state.chronicle.at(-1)?.seq ?? 0;
  if (!world || chronicleBusy || head <= have) return;
  chronicleBusy = true;
  try {
    const { entries } = await client.chronicle(have, 256);
    if (historyWorld !== key) return;
    const known = store.state.chronicle.at(-1)?.seq ?? 0;
    store.update({ chronicle: store.state.chronicle.concat(entries.filter((e) => e.seq > known)) });
  } catch (e) {
    console.warn(`tce: the chronicle could not be read: ${String(e)}`);
  } finally {
    chronicleBusy = false;
  }
}

/** `world:revision:day` of the fields on the map. */
let fieldsKey = "";
let fieldsBusy = false;

/** Fetches the fields when one changed stage, and once a simulated day for their progress. */
async function syncFields(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const clock = s?.clock;
  const key =
    world && clock && s.fieldsRev !== 0
      ? `${store.state.epoch}:${world.worldId}:${s.fieldsRev}:${clock.year}-${clock.month}-${clock.day}`
      : "";
  if (key === fieldsKey || fieldsBusy) return;
  if (!key) {
    fieldsKey = "";
    map.setFields([]);
    return;
  }
  fieldsBusy = true;
  try {
    const { fields } = await client.fields();
    fieldsKey = key;
    map.setFields(fields);
  } catch (e) {
    console.warn(`tce: the fields could not be read: ${String(e)}`);
  } finally {
    fieldsBusy = false;
  }
}

/** `world:revision` of the buildings on the map, and when they were last asked for. */
let buildingsKey = "";
let buildingsBusy = false;
let buildingsAskedAt = 0;
/** Least real time between fetches of the buildings while work moves on, milliseconds. */
const BUILDINGS_REFRESH_MS = 500;

/** Fetches the buildings when one was begun or its work moved on (at most twice a second). */
async function syncBuildings(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const worldKey = world ? `${store.state.epoch}:${world.worldId}` : "";
  const key = world && s.buildingsRev !== 0 ? `${worldKey}:${s.buildingsRev}` : "";
  if (key === buildingsKey || buildingsBusy) return;
  if (!key) {
    buildingsKey = "";
    map.setBuildings([]);
    return;
  }
  // A new world's buildings come at once; work moving on is shown at most twice a second.
  const now = performance.now();
  if (buildingsKey.startsWith(`${worldKey}:`) && now - buildingsAskedAt < BUILDINGS_REFRESH_MS) {
    return;
  }
  buildingsBusy = true;
  buildingsAskedAt = now;
  try {
    const { buildings } = await client.buildings();
    buildingsKey = key;
    map.setBuildings(buildings);
  } catch (e) {
    console.warn(`tce: the buildings could not be read: ${String(e)}`);
  } finally {
    buildingsBusy = false;
  }
}

let personBusy = false;
let personAskedAt = 0;

/** Asks the host about the inspected person again (at most once a second unless `force`). */
async function refreshPerson(force: boolean): Promise<void> {
  const selected = store.state.selected;
  if (!selected || personBusy) return;
  if (!force && performance.now() - personAskedAt < PERSON_REFRESH_MS) return;
  personBusy = true;
  personAskedAt = performance.now();
  const id = selected.id;
  try {
    const info = await client.person(id, DECISIONS_SHOWN);
    if (store.state.selected?.id === id) store.update({ selected: { id, info, error: null } });
  } catch (e) {
    const current = store.state.selected;
    if (current?.id === id) {
      const text = e instanceof HostError || e instanceof Error ? e.message : String(e);
      store.update({ selected: { id, info: current.info, error: text } });
    }
  } finally {
    personBusy = false;
  }
}

function select(id: number | null): void {
  map.setSelected(id);
  store.update({ selected: id === null ? null : { id, info: null, error: null } });
  if (id !== null) void refreshPerson(true);
}

map.onSelect = (id) => select(id);

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
  const field = info.field ? ` · field: ${info.field.status}` : "";
  const building = info.building
    ? ` · ${info.building.program.toLowerCase()}: ${info.building.status}`
    : "";
  readout.textContent =
    `${formatDistance(info.xM)} E, ${formatDistance(info.yM)} S · cell ${info.cellX}, ${info.cellY}` +
    `${height} · ${info.water ?? ""}${field}${building}`;
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
  select,
  focusPerson: (id) => {
    select(id);
    const brief = store.state.snapshot?.people.find((p) => p.id === id);
    if (brief) map.centreOn(brief.x, brief.y, 1);
  },
  focusSettlement: (id) => {
    const s = store.state.snapshot?.settlements.find((x) => x.id === id);
    if (s) map.centreOn(s.x, s.y, 0.5);
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
      people: s.snapshot?.people.length ?? 0,
      settlements: s.snapshot?.settlements.map((x) => ({ name: x.name, population: x.population })) ?? [],
      fieldsRev: s.snapshot?.fieldsRev ?? 0,
      buildingsRev: s.snapshot?.buildingsRev ?? 0,
      chronicle: s.chronicle.map((e) => e.spans.map((x) => x.text).join("")),
      selected: s.selected
        ? { id: s.selected.id, name: s.selected.info?.name ?? null, doing: s.selected.info?.doing ?? null }
        : null,
    };
  },
  peopleOnScreen: () => map.peopleOnScreen(),
  select: (id: number | null) => select(id),
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
