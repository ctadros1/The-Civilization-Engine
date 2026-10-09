// The Civilization Engine's web observer: connects to civ-host, shows the world map, its people
// and its panels, and sends the player's commands. The kernel owns truth; this page only shows it
// and asks for changes.

import "./styles.css";

import { buildingWords } from "./buildings.js";
import { depositWords } from "./deposits.js";
import { formatDistance, simDate } from "./format.js";
import { MapView, type PointerInfo } from "./map/view.js";
import { tenureText } from "./fields.js";
import { pathWords } from "./paths.js";
import { HostClient, HostError } from "./net/client.js";
import * as M from "./net/messages.js";
import { Store, initialState, mergeEvents } from "./state.js";
import { bindUi } from "./ui.js";
import { seasonOf } from "./weather.js";

/**
 * The observer socket: this page's host, or `?host=` in its address. A `?token=` in the address
 * is passed on; a host inside Unreal requires it (ADR-0005 §6).
 */
function socketUrl(): string {
  const params = new URLSearchParams(location.search);
  const host = params.get("host");
  const token = params.get("token");
  const query = token ? `?token=${encodeURIComponent(token)}` : "";
  if (host) return `ws://${host}/ws${query}`;
  return `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/ws${query}`;
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

/**
 * `?view=panels`: the panels alone, for a host that draws the world itself, as Unreal does
 * (ADR-0005 §6).
 */
const PANELS_ONLY = new URLSearchParams(location.search).get("view") === "panels";

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
    map.setSeason(seasonOf(snapshot.clock));
    void syncChronicle();
    void syncFields();
    void syncBuildings();
    void syncPaths();
    void syncMarkets();
    void syncFirms();
    void syncWealth();
    void syncWeather();
    void syncKnowledge();
    void syncStanding();
    void syncGovernment();
    void syncOrder();
    void syncDeposits();
    void syncEarthworks();
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
  let ok = false;
  try {
    const { entries } = await client.chronicle(have, 256);
    if (historyWorld !== key) return;
    const known = store.state.chronicle.at(-1)?.seq ?? 0;
    store.update({ chronicle: store.state.chronicle.concat(entries.filter((e) => e.seq > known)) });
    ok = true;
  } catch (e) {
    console.warn(`tce: the chronicle could not be read: ${String(e)}`);
  } finally {
    chronicleBusy = false;
  }
  // Entries may have come while these were on their way, and the snapshot that announced them
  // may be the last one for a while (a paused world sends none).
  if (ok) void syncChronicle();
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
  let ok = false;
  try {
    const { fields } = await client.fields();
    fieldsKey = key;
    map.setFields(fields);
    ok = true;
  } catch (e) {
    console.warn(`tce: the fields could not be read: ${String(e)}`);
  } finally {
    fieldsBusy = false;
  }
  if (ok) void syncFields();
}

/** `world:revision` of the buildings on the map, and when they were last asked for. */
let buildingsKey = "";
let buildingsBusy = false;
let buildingsAskedAt = 0;
/** An ask held back by the rate limit, so the last change is always fetched. */
let buildingsTimer = 0;
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
  const waited = now - buildingsAskedAt;
  if (buildingsKey.startsWith(`${worldKey}:`) && waited < BUILDINGS_REFRESH_MS) {
    if (!buildingsTimer) {
      buildingsTimer = window.setTimeout(() => {
        buildingsTimer = 0;
        void syncBuildings();
      }, BUILDINGS_REFRESH_MS - waited);
    }
    return;
  }
  buildingsBusy = true;
  buildingsAskedAt = now;
  let ok = false;
  try {
    const { buildings } = await client.buildings();
    buildingsKey = key;
    map.setBuildings(buildings);
    ok = true;
  } catch (e) {
    console.warn(`tce: the buildings could not be read: ${String(e)}`);
  } finally {
    buildingsBusy = false;
  }
  if (ok) void syncBuildings();
}

/** `world:revision` of the deposits on the map. */
let depositsKey = "";
let depositsBusy = false;

/** Fetches the deposits when one was laid down, found or dug from. */
async function syncDeposits(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const goods = store.state.welcome?.goods ?? [];
  const key = world && s.depositsRev !== 0 ? `${store.state.epoch}:${world.worldId}:${s.depositsRev}` : "";
  if (key === depositsKey || depositsBusy) return;
  if (!key) {
    depositsKey = "";
    map.setDeposits([], goods);
    return;
  }
  depositsBusy = true;
  let ok = false;
  try {
    const { deposits } = await client.deposits();
    depositsKey = key;
    map.setDeposits(deposits, goods);
    ok = true;
  } catch (e) {
    console.warn(`tce: the deposits could not be read: ${String(e)}`);
  } finally {
    depositsBusy = false;
  }
  if (ok) void syncDeposits();
}

/** `world:revision` of the earthworks on the map. */
let earthworksKey = "";
let earthworksBusy = false;

/** Fetches the earthworks when one was begun or advanced, and with them the ground they changed. */
async function syncEarthworks(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const key = world && s.earthworksRev !== 0 ? `${store.state.epoch}:${world.worldId}:${s.earthworksRev}` : "";
  if (key === earthworksKey || earthworksBusy) return;
  if (!key) {
    earthworksKey = "";
    map.setEarthworks(null);
    return;
  }
  earthworksBusy = true;
  let ok = false;
  try {
    const info = await client.earthworks();
    earthworksKey = key;
    map.setEarthworks(info);
    ok = true;
  } catch (e) {
    console.warn(`tce: the earthworks could not be read: ${String(e)}`);
  } finally {
    earthworksBusy = false;
  }
  if (ok) void syncEarthworks();
}

/** `world:revision` of the paths on the map. */
let pathsKey = "";
let pathsBusy = false;

/** Fetches the worn ground and trails when the host has surveyed them again. */
async function syncPaths(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const key = world && s.pathsRev !== 0 ? `${store.state.epoch}:${world.worldId}:${s.pathsRev}` : "";
  if (key === pathsKey || pathsBusy) return;
  if (!key) {
    pathsKey = "";
    map.setPaths(null);
    return;
  }
  pathsBusy = true;
  let ok = false;
  try {
    const paths = await client.paths();
    pathsKey = key;
    map.setPaths(paths);
    ok = true;
  } catch (e) {
    console.warn(`tce: the paths could not be read: ${String(e)}`);
  } finally {
    pathsBusy = false;
  }
  if (ok) void syncPaths();
}

/** `world:revision` of the markets shown, the world they belong to, and when they were asked for. */
let marketsKey = "";
let marketsWorld = "";
let marketsBusy = false;
let marketsAskedAt = -Infinity;
/** An ask held back by the rate limit, so the last change is always fetched. */
let marketsTimer = 0;
/** Least real time between fetches of the markets while they change, milliseconds. */
const MARKETS_REFRESH_MS = 1000;

/** Fetches the markets when terms were posted, a trade made or a want went unmet. */
async function syncMarkets(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const worldKey = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (worldKey !== marketsWorld) {
    // Another world's markets are not shown while this one's are read.
    marketsWorld = worldKey;
    marketsKey = "";
    marketsAskedAt = -Infinity;
    store.update({ markets: null, marketsError: null });
  }
  if (!world) return;
  const key = s.marketsRev !== 0 ? `${worldKey}:${s.marketsRev}` : `${worldKey}:none`;
  if (key === marketsKey || marketsBusy) return;
  if (s.marketsRev === 0) {
    marketsKey = key;
    store.update({ markets: [], marketsError: null });
    return;
  }
  const now = performance.now();
  const waited = now - marketsAskedAt;
  if (waited < MARKETS_REFRESH_MS) {
    if (!marketsTimer) {
      marketsTimer = window.setTimeout(() => {
        marketsTimer = 0;
        void syncMarkets();
      }, MARKETS_REFRESH_MS - waited);
    }
    return;
  }
  marketsBusy = true;
  marketsAskedAt = now;
  let ok = false;
  try {
    const { markets } = await client.markets();
    if (marketsWorld === worldKey) {
      marketsKey = key;
      store.update({ markets, marketsError: null });
      ok = true;
    }
  } catch (e) {
    if (marketsWorld === worldKey) {
      store.update({ marketsError: e instanceof Error ? e.message : String(e) });
    }
  } finally {
    marketsBusy = false;
  }
  if (ok) void syncMarkets();
}

/** `world:revision` of the workshops shown, the world they belong to, and when they were asked for. */
let firmsKey = "";
let firmsWorld = "";
let firmsBusy = false;
let firmsAskedAt = -Infinity;
/** An ask held back by the rate limit, so the last change is always fetched. */
let firmsTimer = 0;
/** Least real time between fetches of the workshops while they change, milliseconds. */
const FIRMS_REFRESH_MS = 1000;

function errorText(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** Reads the page of workshop `id`, if it is still the one open. */
async function loadFirm(id: number): Promise<void> {
  try {
    const info = await client.firm(id);
    if (store.state.firm?.id === id) store.update({ firm: { id, info, error: null } });
  } catch (e) {
    const open = store.state.firm;
    if (open?.id === id) store.update({ firm: { id, info: open.info, error: errorText(e) } });
  }
}

/**
 * Fetches the workshops, and the page of the one open, when one opened, closed, made, sold, paid
 * or was reviewed.
 */
async function syncFirms(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const worldKey = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (worldKey !== firmsWorld) {
    // Another world's workshops are not shown while this one's are read.
    firmsWorld = worldKey;
    firmsKey = "";
    firmsAskedAt = -Infinity;
    store.update({ firms: null, firmsError: null, firm: null });
  }
  if (!world) return;
  const key = s.firmsRev !== 0 ? `${worldKey}:${s.firmsRev}` : `${worldKey}:none`;
  if (key === firmsKey || firmsBusy) return;
  if (s.firmsRev === 0) {
    firmsKey = key;
    store.update({ firms: [], firmsError: null });
    return;
  }
  const now = performance.now();
  const waited = now - firmsAskedAt;
  if (waited < FIRMS_REFRESH_MS) {
    if (!firmsTimer) {
      firmsTimer = window.setTimeout(() => {
        firmsTimer = 0;
        void syncFirms();
      }, FIRMS_REFRESH_MS - waited);
    }
    return;
  }
  firmsBusy = true;
  firmsAskedAt = now;
  let ok = false;
  try {
    const { firms } = await client.firms();
    if (firmsWorld === worldKey) {
      firmsKey = key;
      store.update({ firms, firmsError: null });
      ok = true;
      const open = store.state.firm;
      if (open) await loadFirm(open.id);
    }
  } catch (e) {
    if (firmsWorld === worldKey) store.update({ firmsError: errorText(e) });
  } finally {
    firmsBusy = false;
  }
  if (ok) void syncFirms();
}

/** `world:revision` of the wealth measures shown, the world they belong to, and when they were asked for. */
let wealthKey = "";
let wealthWorld = "";
let wealthBusy = false;
let wealthAskedAt = -Infinity;
/** An ask held back by the rate limit, so the last change is always fetched. */
let wealthTimer = 0;
/** Least real time between fetches of the wealth measures, milliseconds. */
const WEALTH_REFRESH_MS = 2000;

/**
 * Fetches the wealth measures at the start of each month, when households form or end, and when
 * a year's measures are recorded.
 */
async function syncWealth(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const worldKey = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (worldKey !== wealthWorld) {
    // Another world's measures are not shown while this one's are read.
    wealthWorld = worldKey;
    wealthKey = "";
    wealthAskedAt = -Infinity;
    store.update({ wealth: null, wealthError: null });
  }
  if (!world) return;
  const key = `${worldKey}:${s.wealthRev}`;
  if (key === wealthKey || wealthBusy) return;
  const now = performance.now();
  const waited = now - wealthAskedAt;
  if (waited < WEALTH_REFRESH_MS) {
    if (!wealthTimer) {
      wealthTimer = window.setTimeout(() => {
        wealthTimer = 0;
        void syncWealth();
      }, WEALTH_REFRESH_MS - waited);
    }
    return;
  }
  wealthBusy = true;
  wealthAskedAt = now;
  let ok = false;
  try {
    const wealth = await client.wealth();
    if (wealthWorld === worldKey) {
      wealthKey = key;
      store.update({ wealth, wealthError: null });
      ok = true;
    }
  } catch (e) {
    if (wealthWorld === worldKey) store.update({ wealthError: errorText(e) });
  } finally {
    wealthBusy = false;
  }
  if (ok) void syncWealth();
}

/** `world:revision` of the weather shown, the world it belongs to, and when it was asked for. */
let weatherKey = "";
let weatherWorld = "";
let weatherBusy = false;
let weatherAskedAt = -Infinity;
/** An ask held back by the rate limit, so the last change is always fetched. */
let weatherTimer = 0;
/** Least real time between fetches of the weather: the record changes every day lived. */
const WEATHER_REFRESH_MS = 2000;

/** Fetches every month's weather as days are lived (wire 1.24). */
async function syncWeather(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const worldKey = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (worldKey !== weatherWorld) {
    // Another world's weather is not shown while this one's is read.
    weatherWorld = worldKey;
    weatherKey = "";
    weatherAskedAt = -Infinity;
    store.update({ weather: null, weatherError: null });
  }
  if (!world || !s.weatherRev) return;
  const key = `${worldKey}:${s.weatherRev}`;
  if (key === weatherKey || weatherBusy) return;
  const now = performance.now();
  const waited = now - weatherAskedAt;
  if (waited < WEATHER_REFRESH_MS) {
    if (!weatherTimer) {
      weatherTimer = window.setTimeout(() => {
        weatherTimer = 0;
        void syncWeather();
      }, WEATHER_REFRESH_MS - waited);
    }
    return;
  }
  weatherBusy = true;
  weatherAskedAt = now;
  let ok = false;
  try {
    const weather = await client.weather();
    if (weatherWorld === worldKey) {
      weatherKey = key;
      store.update({ weather, weatherError: null });
      ok = true;
    }
  } catch (e) {
    if (weatherWorld === worldKey) store.update({ weatherError: errorText(e) });
  } finally {
    weatherBusy = false;
  }
  if (ok) void syncWeather();
}

/** `world:year-month` of the standing shown, and the world it belongs to. */
let standingKey = "";
let standingWorld = "";
let standingBusy = false;

/**
 * Fetches each settlement's standing once a month: the kernel works it out on the first of each
 * month (ADR-0014 §3), so a new month is when it can have changed.
 */
async function syncStanding(): Promise<void> {
  const s = store.state;
  const world = s.snapshot?.world;
  const clock = s.snapshot?.clock;
  if (!world || !clock) return;
  const worldKey = world.worldId;
  if (worldKey !== standingWorld) {
    standingWorld = worldKey;
    standingKey = "";
    store.update({ standing: null, standingError: null });
  }
  const d = simDate(clock.minute);
  const key = `${worldKey}:${d.year}-${d.month}`;
  if (key === standingKey || standingBusy) return;
  standingBusy = true;
  try {
    const standing = await client.standing();
    if (standingWorld === worldKey) {
      standingKey = key;
      store.update({ standing, standingError: null });
    }
  } catch (e) {
    if (standingWorld === worldKey) store.update({ standingError: errorText(e) });
  } finally {
    standingBusy = false;
  }
}

/** `world:year-month-day` of the government shown, and the world it belongs to. */
let governmentKey = "";
let governmentWorld = "";
let governmentBusy = false;

/**
 * Fetches each settlement's polity once a day: proposals, gatherings and their decisions happen
 * at midnight (ADR-0013), and levies and asks during the day are tallied in the next fetch.
 */
async function syncGovernment(): Promise<void> {
  const s = store.state;
  const world = s.snapshot?.world;
  const clock = s.snapshot?.clock;
  if (!world || !clock) return;
  const worldKey = world.worldId;
  if (worldKey !== governmentWorld) {
    governmentWorld = worldKey;
    governmentKey = "";
    store.update({ government: null, governmentError: null });
  }
  const d = simDate(clock.minute);
  const key = `${worldKey}:${d.year}-${d.month}-${d.day}`;
  if (key === governmentKey || governmentBusy) return;
  governmentBusy = true;
  try {
    const government = await client.government();
    if (governmentWorld === worldKey) {
      governmentKey = key;
      store.update({ government, governmentError: null });
    }
  } catch (e) {
    if (governmentWorld === worldKey) store.update({ governmentError: errorText(e) });
  } finally {
    governmentBusy = false;
  }
}

/** `world:year-month-day` of the takings shown, and the world they belong to. */
let orderKey = "";
let orderWorld = "";
let orderBusy = false;

/**
 * Fetches the takings once a day: losses are found, word goes round and households choose at
 * midnight (ADR-0015); the day's takings are in the next fetch.
 */
async function syncOrder(): Promise<void> {
  const s = store.state;
  const world = s.snapshot?.world;
  const clock = s.snapshot?.clock;
  if (!world || !clock) return;
  const worldKey = world.worldId;
  if (worldKey !== orderWorld) {
    orderWorld = worldKey;
    orderKey = "";
    store.update({ order: null, orderError: null });
  }
  const d = simDate(clock.minute);
  const key = `${worldKey}:${d.year}-${d.month}-${d.day}`;
  if (key === orderKey || orderBusy) return;
  orderBusy = true;
  try {
    const order = await client.order();
    if (orderWorld === worldKey) {
      orderKey = key;
      store.update({ order, orderError: null });
    }
  } catch (e) {
    if (orderWorld === worldKey) store.update({ orderError: errorText(e) });
  } finally {
    orderBusy = false;
  }
}

/** `world:revision` of the knowledge shown, the world it belongs to, and when it was asked for. */
let knowledgeKey = "";
let knowledgeWorld = "";
let knowledgeBusy = false;
let knowledgeAskedAt = -Infinity;
/** An ask held back by the rate limit, so the last change is always fetched. */
let knowledgeTimer = 0;
/** Least real time between fetches of the knowledge, milliseconds. */
const KNOWLEDGE_REFRESH_MS = 2000;

/**
 * Fetches what each settlement knows whenever someone comes to know, learns toward, hears of or
 * loses a technique, and when people arrive, leave or die.
 */
async function syncKnowledge(): Promise<void> {
  const s = store.state.snapshot;
  const world = s?.world;
  const worldKey = world ? `${store.state.epoch}:${world.worldId}` : "";
  if (worldKey !== knowledgeWorld) {
    // Another world's knowledge is not shown while this one's is read.
    knowledgeWorld = worldKey;
    knowledgeKey = "";
    knowledgeAskedAt = -Infinity;
    store.update({ knowledge: null, knowledgeError: null });
  }
  if (!world) return;
  const key = `${worldKey}:${s.knowledgeRev}`;
  if (key === knowledgeKey || knowledgeBusy) return;
  const now = performance.now();
  const waited = now - knowledgeAskedAt;
  if (waited < KNOWLEDGE_REFRESH_MS) {
    if (!knowledgeTimer) {
      knowledgeTimer = window.setTimeout(() => {
        knowledgeTimer = 0;
        void syncKnowledge();
      }, KNOWLEDGE_REFRESH_MS - waited);
    }
    return;
  }
  knowledgeBusy = true;
  knowledgeAskedAt = now;
  let ok = false;
  try {
    const knowledge = await client.knowledge();
    if (knowledgeWorld === worldKey) {
      knowledgeKey = key;
      store.update({ knowledge, knowledgeError: null });
      ok = true;
    }
  } catch (e) {
    if (knowledgeWorld === worldKey) store.update({ knowledgeError: errorText(e) });
  } finally {
    knowledgeBusy = false;
  }
  if (ok) void syncKnowledge();
}

/**
 * The observer introduces a technique to someone (god tool, ADR-0008 §6): they come to know it,
 * or with `awareOnly` only hear of it. The inspector and the knowledge panel are asked again.
 */
async function introduceTechnique(
  person: number,
  technique: number,
  awareOnly: boolean,
): Promise<void> {
  try {
    const body = await client.command(M.introduceTechnique(person, technique, awareOnly));
    const text = body.kind === "ack" && body.message ? body.message : "Introduced";
    store.update({ notice: { kind: "info", text } });
  } catch (e) {
    store.update({ notice: { kind: "error", text: errorText(e) } });
  }
  void refreshPerson(true);
}

/**
 * The observer whispers a true claim to someone, or tells them of an ideology (god tools, M4c
 * slice AJ, ADR-0016 §5). The kernel's reply says what reached them; the inspector is asked again.
 */
async function influence(payload: Uint8Array, fallback: string): Promise<void> {
  try {
    const body = await client.command(payload);
    const text = body.kind === "ack" && body.message ? body.message : fallback;
    store.update({ notice: { kind: "info", text } });
  } catch (e) {
    store.update({ notice: { kind: "error", text: errorText(e) } });
  }
  void refreshPerson(true);
}

const whisper = (person: number, claim: number): Promise<void> =>
  influence(M.whisper(person, claim), "Whispered");
const bless = (person: number, curse: boolean, days: number, share: number): Promise<void> =>
  influence(M.bless(person, curse, days, share), curse ? "Cursed" : "Blessed");
const tellOfIdeology = (person: number, ideology: number): Promise<void> =>
  influence(M.tellOfIdeology(person, ideology), "Told");

/** Opens workshop `id`'s page in the workshops panel, or goes back to the list. */
function openFirm(id: number | null): void {
  store.update({ firm: id === null ? null : { id, info: null, error: null } });
  if (id !== null) void loadFirm(id);
}

let personBusy = false;
let personAskedAt = 0;
/** Another ask is due once the one on its way is answered. */
let personAgain = false;
/** An ask held back by the rate limit, so the inspector never stops on a stale answer. */
let personTimer = 0;

/** Asks the host about the inspected person again (at most once a second unless `force`). */
async function refreshPerson(force: boolean): Promise<void> {
  const selected = store.state.selected;
  if (!selected) return;
  if (personBusy) {
    personAgain = true;
    return;
  }
  const waited = performance.now() - personAskedAt;
  if (!force && waited < PERSON_REFRESH_MS) {
    if (!personTimer) {
      personTimer = window.setTimeout(() => {
        personTimer = 0;
        void refreshPerson(false);
      }, PERSON_REFRESH_MS - waited);
    }
    return;
  }
  personBusy = true;
  personAgain = false;
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
  // Someone else was chosen, or the world moved on, while this ask was on its way.
  if (personAgain) void refreshPerson(store.state.selected?.id !== id);
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

/** The households with someone living in them, from the latest snapshot. */
function livingHouseholds(): Set<number> {
  return new Set(store.state.snapshot?.people.map((p) => p.household) ?? []);
}

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
  const field = info.field
    ? ` · field: ${info.field.status}; ${tenureText(info.field, livingHouseholds())}`
    : "";
  const path = info.path && !info.building ? ` · ${pathWords(info.path)}` : "";
  const building = info.building ? ` · ${buildingWords(info.building)}` : "";
  const deposit = info.deposit ? ` · ${depositWords(info.deposit, store.state.welcome?.goods ?? [])}` : "";
  const earthwork = info.earthwork ? ` · ${info.earthwork.words}` : "";
  readout.textContent =
    `${formatDistance(info.xM)} E, ${formatDistance(info.yM)} S · cell ${info.cellX}, ${info.cellY}` +
    `${height} · ${info.water ?? ""}${path}${field}${building}${earthwork}${deposit}`;
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
    if (brief) lookAt(brief.x, brief.y, 1);
  },
  focusSettlement: (id) => {
    const s = store.state.snapshot?.settlements.find((x) => x.id === id);
    if (s) lookAt(s.x, s.y, 0.5);
  },
  openFirm,
  runAhead: async (minutes) => {
    const clock = store.state.snapshot?.clock;
    if (clock) await command(M.runUntil(clock.minute + minutes));
  },
  setPlacing,
  setPlacingDeposit,
  introduceTechnique,
  whisper,
  tellOfIdeology,
  bless,
  setPlacingAgitator,
  setPlacingWave,
});

/**
 * Arms or disarms the map tool that sends families where the map is clicked, and sets how many
 * come together.
 */
function setPlacing(on: boolean, families = store.state.placeFamilies): void {
  store.update({ placing: on, placeFamilies: families, placingDeposit: false, placingAgitator: false, placingWave: false });
  map.setPlacing(on);
}

/**
 * Arms or disarms the map tool that sends an agitator holding `ideology` (an index into
 * Welcome.ideologies) where the map is clicked (M4c slice AJ).
 */
function setPlacingAgitator(on: boolean, ideology: number): void {
  store.update({ placingAgitator: on, agitatorIdeology: ideology, placing: false, placingDeposit: false, placingWave: false });
  map.setPlacing(on);
}

/**
 * Arms or disarms the map tool that sends a migration wave where the map is clicked (M5a slice
 * AN): `households` households over `days` days, each carrying `months` months of food.
 */
function setPlacingWave(on: boolean, households: number, days: number, months: number): void {
  store.update({
    placingWave: on,
    waveHouseholds: households,
    waveDays: days,
    waveMonths: months,
    placing: false,
    placingDeposit: false,
    placingAgitator: false,
  });
  map.setPlacing(on);
}

/**
 * Arms or disarms the map tool that lays down a deposit of `good` (a content id) where the map is
 * clicked, showing at the surface or buried (M3b slice Q).
 */
function setPlacingDeposit(on: boolean, good: string, exposed: boolean): void {
  store.update({
    placingDeposit: on,
    depositGood: good,
    depositExposed: exposed,
    placing: false,
    placingAgitator: false,
    placingWave: false,
  });
  map.setPlacing(on);
}

/** The radius, metres, of a deposit the observer lays down. */
const PLACED_DEPOSIT_RADIUS_M = 10;

map.onPlace = (xM, yM) => {
  if (store.state.placingWave) {
    const { waveHouseholds, waveDays, waveMonths } = store.state;
    setPlacingWave(false, waveHouseholds, waveDays, waveMonths);
    void influence(M.sendWave(xM, yM, waveHouseholds, waveDays, waveMonths), "A wave was sent");
    return;
  }
  if (store.state.placingAgitator) {
    const ideology = store.state.agitatorIdeology;
    setPlacingAgitator(false, ideology);
    void influence(M.sendAgitator(xM, yM, ideology), "An agitator was sent");
    return;
  }
  if (store.state.placingDeposit) {
    const { depositGood, depositExposed } = store.state;
    setPlacingDeposit(false, depositGood, depositExposed);
    void (async () => {
      try {
        const body = await client.command(
          M.placeDeposit(xM, yM, depositGood, PLACED_DEPOSIT_RADIUS_M, depositExposed),
        );
        const text = body.kind === "ack" && body.message ? body.message : "A deposit was laid down";
        store.update({ notice: { kind: "info", text } });
      } catch (e) {
        const text = e instanceof HostError ? e.message : String(e);
        store.update({ notice: { kind: "error", text } });
      }
    })();
    return;
  }
  setPlacing(false);
  void (async () => {
    try {
      const body = await client.command(M.spawnFamily(xM, yM, store.state.placeFamilies));
      const text = body.kind === "ack" && body.message ? body.message : "A family arrived";
      store.update({ notice: { kind: "info", text } });
    } catch (e) {
      const text = e instanceof HostError ? e.message : String(e);
      store.update({ notice: { kind: "error", text } });
    }
  })();
};

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && store.state.placing) setPlacing(false);
  if (event.key === "Escape" && store.state.placingDeposit) {
    setPlacingDeposit(false, store.state.depositGood, store.state.depositExposed);
  }
  if (event.key === "Escape" && store.state.placingAgitator) {
    setPlacingAgitator(false, store.state.agitatorIdeology);
  }
  if (event.key === "Escape" && store.state.placingWave) {
    const { waveHouseholds, waveDays, waveMonths } = store.state;
    setPlacingWave(false, waveHouseholds, waveDays, waveMonths);
  }
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
      marketsRev: s.snapshot?.marketsRev ?? 0,
      firmsRev: s.snapshot?.firmsRev ?? 0,
      wealthRev: s.snapshot?.wealthRev ?? 0,
      knowledgeRev: s.snapshot?.knowledgeRev ?? 0,
      depositsRev: s.snapshot?.depositsRev ?? 0,
      earthworksRev: s.snapshot?.earthworksRev ?? 0,
      weatherRev: s.snapshot?.weatherRev ?? 0,
      weather: s.weather
        ? {
            months: s.weather.months.length,
            today: s.weather.today?.words ?? null,
            snowLineM: s.weather.today?.snowLineM ?? null,
            annualMm: s.weather.annualMm,
          }
        : null,
      government: s.government
        ? {
            minute: s.government.minute,
            polities: s.government.polities.map((p) => ({
              name: p.name,
              members: p.members,
              custom: p.custom,
              store: p.store,
              offices: p.offices,
              label: p.label,
              labelWhy: p.labelWhy,
              gatheringLaw: p.gatheringLaw,
              gatheringCases: p.gatheringCases,
              customHistory: p.customHistory,
              revolts: p.revolts,
              coups: p.coups,
              petitions: p.petitions,
              refusals: p.refusals,
              laws: p.laws.map((l) => ({
                what: l.what,
                status: l.status,
                outcome: l.outcome,
                decision: l.decision,
                stances: l.stances.length,
                known: l.known,
              })),
            })),
          }
        : null,
      order: s.order
        ? {
            minute: s.order.minute,
            attempts: s.order.attempts,
            takings: s.order.takings,
            seen: s.order.seen,
            demands: s.order.demands,
            refusals: s.order.refusals,
            cases: s.order.cases,
            found: s.order.found,
            incidents: s.order.incidents.map((i) => ({
              id: i.id,
              outcome: i.outcome,
              actor: i.actorName,
              target: i.targetName,
              seenBy: i.seenBy.length,
              watch: i.watch,
            })),
            known: s.order.known.map((k) => ({
              incident: k.incident,
              knowTaker: k.knowTaker,
              sources: k.sources,
              response: k.response,
              owed: k.owed,
              case: k.case,
            })),
          }
        : null,
      standing: s.standing
        ? {
            minute: s.standing.minute,
            ties: s.standing.ties,
            letGo: s.standing.letGo,
            settlements: s.standing.settlements.map((x) => ({
              name: x.name,
              adults: x.adults,
              notables: x.rows.filter((r) => r.notable).map((r) => r.name),
              rows: x.rows.length,
            })),
          }
        : null,
      knowledge: s.knowledge
        ? s.knowledge.settlements.map((x) => ({
            name: x.name,
            techniques: x.techniques.map((t) => ({
              id: s.welcome?.techniques[t.technique]?.id ?? "",
              known: t.known,
              knowers: t.knowers.length,
              learners: t.learners.length,
              heard: t.heard.length,
              status: t.status,
              history: t.history,
            })),
          }))
        : null,
      wealth: s.wealth
        ? {
            regime: s.wealth.regimeName,
            settlements: s.wealth.settlements.map((x) => ({
              name: x.name,
              people: x.now?.people ?? 0,
              households: x.households.length,
              history: x.history.map((y) => y.year),
              holdingNone: x.now?.holdingNone ?? null,
              commonHa: x.now?.commonHa ?? null,
            })),
          }
        : null,
      firms:
        s.firms?.map((f) => ({
          id: f.id,
          name: f.name,
          open: f.open,
          record: f.record,
          hiringH: f.hiringH,
        })) ?? null,
      firm: s.firm
        ? {
            id: s.firm.id,
            name: s.firm.info?.brief.name ?? null,
            entries: s.firm.info?.entries.length ?? 0,
            months: s.firm.info?.months.length ?? 0,
            wage: s.firm.info?.wage?.text ?? null,
            error: s.firm.error,
          }
        : null,
      markets:
        s.markets?.map((m) => ({
          settlement: m.settlementName,
          money: m.money,
          goods: m.goods.length,
          offers: m.offers.length,
          trades: m.recent.length,
        })) ?? null,
      chronicle: s.chronicle.map((e) => e.spans.map((x) => x.text).join("")),
      selected: s.selected
        ? {
            id: s.selected.id,
            name: s.selected.info?.name ?? null,
            doing: s.selected.info?.doing ?? null,
            untilMinute: s.selected.info?.untilMinute ?? null,
            knows:
              s.selected.info?.knows.map((k) => ({
                id: s.welcome?.techniques[k.technique]?.id ?? "",
                state: k.state,
                source: k.source,
              })) ?? null,
          }
        : null,
    };
  },
  peopleOnScreen: () => map.peopleOnScreen(),
  briefs: () =>
    store.state.snapshot?.people.map((p) => ({ id: p.id, sex: p.sex, ageYears: p.ageYears })) ?? [],
  select: (id: number | null) => select(id),
  openFirm: (id: number | null) => openFirm(id),
  /** Introduces a technique, by content id, to a person (god tool). */
  introduceTechnique: (person: number, id: string, awareOnly: boolean) => {
    const technique = store.state.welcome?.techniques.findIndex((t) => t.id === id) ?? -1;
    return introduceTechnique(person, technique < 0 ? 0xffff : technique, awareOnly);
  },
  /** Tells a person of an ideology, by content id (god tool). */
  tellOfIdeology: (person: number, id: string) => {
    const k = store.state.welcome?.ideologies.findIndex((d) => d.id === id) ?? -1;
    return tellOfIdeology(person, k < 0 ? 0xffff : k);
  },
  /** Whispers a claim to a person (god tool). */
  whisper: (person: number, claim: number) => whisper(person, claim),
  /** Blesses or curses a person (god tool). */
  bless: (person: number, curse: boolean, days: number, share: number) => bless(person, curse, days, share),
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

/**
 * Shows a place: centres the map on it, or, with the panels alone, asks the host drawing the
 * world to look there with a `tce:focus` event on `window` (detail: metres east and south).
 */
function lookAt(xM: number, yM: number, minScale: number): void {
  if (PANELS_ONLY) window.dispatchEvent(new CustomEvent("tce:focus", { detail: { x: xM, y: yM } }));
  else map.centreOn(xM, yM, minScale);
}

async function start(): Promise<void> {
  if (PANELS_ONLY) {
    // The host draws the world itself; the panels need no map or WebGL.
    document.body.classList.add("panels-only");
    store.update({ mapAvailable: false });
    client.start();
    return;
  }
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
