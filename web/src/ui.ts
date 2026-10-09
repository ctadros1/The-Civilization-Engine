// The shell's panels and dialogs. Every panel has empty, loading and error states (plan §7).
// Rendering is plain DOM: text and visibility are set from the store on every change.

import {
  formatAge,
  formatBytes,
  formatClockTime,
  formatDays,
  formatDistance,
  formatKg,
  formatMapSize,
  formatPercent,
  formatPersonAge,
  formatPoints,
  formatSimDate,
  formatSimMinute,
  formatSpeed,
  formatTools,
  formatWhen,
} from "./format.js";
import { BUILDING_LEGEND } from "./buildings.js";
import {
  amountText,
  askingText,
  monthIndex,
  monthText,
  offerGroups,
  paymentsText,
  priceSeries,
  priceText,
  sparkline,
  worthText,
} from "./market.js";
import { FIELD_LEGEND } from "./fields.js";
import {
  ENTRIES_SHOWN,
  holdingsText,
  lifeText,
  linesText,
  statementRows,
} from "./firm.js";
import { materialGoods } from "./deposits.js";
import { introducible, knowRows, summaryText, techniqueRows } from "./knowledge.js";
import { standingText, tieText } from "./standing.js";
import {
  claimText,
  grievanceFacts,
  grievanceHead,
  heardHow,
  heardWhen,
  factionText,
  ideologyText,
  normText,
  positionText,
  valuesText,
} from "./word.js";
import {
  brokenText,
  labelText,
  lawDayText,
  levyText,
  membersText,
  reliefText,
  stanceText,
  statusText,
} from "./government.js";
import { believedText, caseText, choiceText, happenedRest, seenText, totalsText } from "./order.js";
import { PATH_LEGEND } from "./paths.js";
import { neighbourSizes } from "./settlements.js";
import {
  HOUSEHOLDS_SHOWN,
  giniLines,
  householdRows,
  measureRows,
  spreadText,
  yearRows,
} from "./wealth.js";
import { monthRows, yearRows as weatherYearRows } from "./weather.js";
import { HostError } from "./net/client.js";
import type {
  ChronicleEntry,
  Decision,
  EventItem,
  FirmBrief,
  MarketInfo,
  PersonInfo,
  SaveEntry,
  ScoredOption,
  SettlementKnowledge,
  SettlementWealth,
  TechniqueInfo,
  Welcome,
  WorldInfo,
} from "./net/messages.js";
import { activityColour, cssColour } from "./people.js";
import type { AppState, FirmPage, Selection, Store } from "./state.js";

export interface Actions {
  newWorld(args: {
    name: string;
    presetId: string;
    sizeCells: number;
    seed: bigint;
    bandSize: number;
    /** The property regime, by content id; empty = the content's default. */
    regimeId: string;
    /** People in each further founding group, placed together with the first (ADR-0018). */
    neighbours: number[];
    /** The founding groups know where each other camped (ADR-0018 §6). */
    neighboursKnown: boolean;
  }): Promise<void>;
  save(label: string): Promise<void>;
  listSaves(): Promise<SaveEntry[]>;
  load(file: string): Promise<void>;
  setClock(paused: boolean, speed: number): Promise<void>;
  cancelTask(): Promise<void>;
  recover(accept: boolean): Promise<void>;
  retryMap(): void;
  /** Inspect a person (null closes the inspector). */
  select(id: number | null): void;
  /** Inspect a person and centre the map on them. */
  focusPerson(id: number): void;
  /** Centre the map on a settlement. */
  focusSettlement(id: number): void;
  /** Open a workshop's page in the workshops panel (null goes back to the list). */
  openFirm(id: number | null): void;
  /** Run ahead by `minutes` of simulated time. */
  runAhead(minutes: number): Promise<void>;
  /**
   * Arm or disarm the map tool that sends families where the map is clicked, and set how many
   * come together.
   */
  setPlacing(on: boolean, families: number): void;
  /** Arms or disarms the tool that lays down a deposit of a good where the map is clicked. */
  setPlacingDeposit(on: boolean, good: string, exposed: boolean): void;
  /**
   * Introduce a technique to a living person (god tool): they come to know it, or with
   * `awareOnly` only hear of it. `technique` is an index into Welcome.techniques.
   */
  introduceTechnique(person: number, technique: number, awareOnly: boolean): Promise<void>;
  /** Whisper a true claim to a living adult (god tool, M4c slice AJ): a number from their news. */
  whisper(person: number, claim: number): Promise<void>;
  /** Tell a living adult of an ideology (god tool, M4c slice AJ): an index into
   * Welcome.ideologies. */
  tellOfIdeology(person: number, ideology: number): Promise<void>;
  /** Bless a living person, or with `curse` curse them, for `days` days by `share` (god tool). */
  bless(person: number, curse: boolean, days: number, share: number): Promise<void>;
  /** Arms or disarms the tool that sends an agitator holding ideology `ideology` where the map is
   * clicked (M4c slice AJ). */
  setPlacingAgitator(on: boolean, ideology: number): void;
  /** Arms or disarms the tool that sends a migration wave of `households` households over `days`
   * days, each carrying `months` months of food, where the map is clicked (M5a slice AN). */
  setPlacingWave(on: boolean, households: number, days: number, months: number): void;
}

const MAX_SEED = (1n << 64n) - 1n;

function $<T extends HTMLElement = HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el as T;
}

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: { className?: string; text?: string } = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (props.className) node.className = props.className;
  if (props.text !== undefined) node.textContent = props.text;
  node.append(...children);
  return node;
}

function setText(node: HTMLElement, text: string): void {
  if (node.textContent !== text) node.textContent = text;
}

function errorText(e: unknown): string {
  if (e instanceof HostError) return e.message;
  return e instanceof Error ? e.message : String(e);
}

function randomSeed(): bigint {
  const words = new Uint32Array(1);
  crypto.getRandomValues(words);
  return BigInt(words[0]!);
}

function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLTextAreaElement
  );
}

function presetName(welcome: Welcome | null, id: string): string {
  return welcome?.presets.find((p) => p.id === id)?.name ?? id;
}

export function bindUi(store: Store, actions: Actions): void {
  const notify = (text: string, kind: "error" | "info" = "error") => {
    store.update({ notice: { kind, text } });
    if (kind === "info") {
      setTimeout(() => {
        if (store.state.notice?.text === text) store.update({ notice: null });
      }, 5000);
    }
  };
  const attempt = async (work: () => Promise<void>) => {
    try {
      await work();
    } catch (e) {
      notify(errorText(e));
    }
  };

  // ---- New world --------------------------------------------------------------------------
  const nwDialog = $<HTMLDialogElement>("new-world-dialog");
  const nwForm = $<HTMLFormElement>("new-world-form");
  const nwPreset = $<HTMLSelectElement>("nw-preset");
  const nwSize = $<HTMLSelectElement>("nw-size");
  const nwSeed = $<HTMLInputElement>("nw-seed");
  const nwBand = $<HTMLInputElement>("nw-band");
  const nwNeighbours = $<HTMLInputElement>("nw-neighbours");
  const nwKnown = $<HTMLInputElement>("nw-neighbours-known");
  const knownApplies = () => {
    nwKnown.disabled = nwNeighbours.value.trim() === "";
  };
  nwNeighbours.addEventListener("input", knownApplies);
  const nwRegime = $<HTMLSelectElement>("nw-regime");
  const nwError = $("nw-error");
  const nwCreate = $<HTMLButtonElement>("nw-create");
  let filledFor: Welcome | null = null;

  const describePreset = () => {
    const preset = store.state.welcome?.presets.find((p) => p.id === nwPreset.value);
    setText($("nw-preset-description"), preset?.description ?? "");
  };
  const describeRegime = () => {
    const regime = store.state.welcome?.regimes.find((r) => r.id === nwRegime.value);
    setText($("nw-regime-description"), regime?.description ?? "");
    $("nw-regime-rules").replaceChildren(
      ...(regime?.rules ?? []).map((rule) => el("li", { text: rule })),
    );
  };
  const fillNewWorld = (welcome: Welcome) => {
    if (filledFor === welcome) return;
    filledFor = welcome;
    nwPreset.replaceChildren(
      ...welcome.presets.map((p) => {
        const option = el("option", { text: p.name });
        option.value = p.id;
        option.selected = p.isDefault;
        return option;
      }),
    );
    nwSize.replaceChildren(
      ...welcome.mapSizes.map((cells) => {
        const option = el("option", { text: formatMapSize(cells, welcome.cellSizeM) });
        option.value = String(cells);
        option.selected = cells === welcome.defaultMapSize;
        return option;
      }),
    );
    nwRegime.replaceChildren(
      ...welcome.regimes.map((r) => {
        const option = el("option", { text: r.name });
        option.value = r.id;
        option.selected = r.isDefault;
        return option;
      }),
    );
    nwRegime.disabled = welcome.regimes.length === 0;
    nwBand.min = String(welcome.bandSizeMin);
    nwBand.max = String(welcome.bandSizeMax);
    setText(
      $("nw-band-hint"),
      `People in the first band, ${welcome.bandSizeMin} to ${welcome.bandSizeMax}. They choose where to camp.`,
    );
    setText(
      $("nw-neighbours-hint"),
      `Up to two more bands, each with its own people and way of building, camping where their ` +
        `fields lie apart from the others'. Their sizes, separated by commas; blank for none.`,
    );
    describePreset();
    describeRegime();
  };
  nwPreset.addEventListener("change", describePreset);
  nwRegime.addEventListener("change", describeRegime);
  $("nw-seed-random").addEventListener("click", () => {
    nwSeed.value = randomSeed().toString();
  });
  const openNewWorld = () => {
    const welcome = store.state.welcome;
    if (!welcome) return;
    fillNewWorld(welcome);
    nwSeed.value = randomSeed().toString();
    nwBand.value = String(welcome.bandSizeDefault);
    nwNeighbours.value = "";
    nwKnown.checked = false;
    knownApplies();
    nwError.hidden = true;
    nwCreate.disabled = false;
    nwDialog.showModal();
    $<HTMLInputElement>("nw-name").focus();
  };
  $("nw-cancel").addEventListener("click", () => nwDialog.close());
  nwForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const text = nwSeed.value.trim();
    if (!/^\d{1,20}$/.test(text) || BigInt(text) > MAX_SEED) {
      nwError.textContent = `The seed must be a whole number from 0 to ${MAX_SEED}.`;
      nwError.hidden = false;
      return;
    }
    const welcome = store.state.welcome;
    const band = Number(nwBand.value);
    if (
      welcome &&
      (!Number.isInteger(band) || band < welcome.bandSizeMin || band > welcome.bandSizeMax)
    ) {
      nwError.textContent = `The founding band must have ${welcome.bandSizeMin} to ${welcome.bandSizeMax} people.`;
      nwError.hidden = false;
      return;
    }
    const neighbours = neighbourSizes(nwNeighbours.value, welcome);
    if (typeof neighbours === "string") {
      nwError.textContent = neighbours;
      nwError.hidden = false;
      return;
    }
    nwCreate.disabled = true;
    actions
      .newWorld({
        name: $<HTMLInputElement>("nw-name").value,
        presetId: nwPreset.value,
        sizeCells: Number(nwSize.value),
        seed: BigInt(text),
        bandSize: welcome ? band : 0,
        regimeId: nwRegime.value,
        neighbours,
        neighboursKnown: neighbours.length > 0 && nwKnown.checked,
      })
      .then(() => nwDialog.close())
      .catch((e: unknown) => {
        nwError.textContent = errorText(e);
        nwError.hidden = false;
      })
      .finally(() => {
        nwCreate.disabled = false;
      });
  });

  // ---- Save -------------------------------------------------------------------------------
  const saveDialog = $<HTMLDialogElement>("save-dialog");
  const saveError = $("save-error");
  const saveSubmit = $<HTMLButtonElement>("save-submit");
  const openSave = () => {
    $<HTMLInputElement>("save-label").value = "";
    saveError.hidden = true;
    saveSubmit.disabled = false;
    saveDialog.showModal();
  };
  $("save-cancel").addEventListener("click", () => saveDialog.close());
  $<HTMLFormElement>("save-form").addEventListener("submit", (event) => {
    event.preventDefault();
    saveSubmit.disabled = true;
    actions
      .save($<HTMLInputElement>("save-label").value)
      .then(() => {
        saveDialog.close();
        notify("Saved.", "info");
      })
      .catch((e: unknown) => {
        saveError.textContent = errorText(e);
        saveError.hidden = false;
      })
      .finally(() => {
        saveSubmit.disabled = false;
      });
  });

  // ---- Load -------------------------------------------------------------------------------
  const loadDialog = $<HTMLDialogElement>("load-dialog");
  const loadBody = $("load-body");
  const renderSaves = (saves: SaveEntry[]) => {
    if (saves.length === 0) {
      loadBody.replaceChildren(
        el(
          "p",
          { className: "empty" },
          "No saves yet. Saves appear here when you save, and when a world is autosaved.",
        ),
      );
      return;
    }
    const head = el(
      "tr",
      {},
      ...["World", "Save", "In-game time", "Written", "Size", ""].map((t) => el("th", { text: t })),
    );
    const rows = saves.map((s) => {
      const button = el("button", { text: "Load" });
      button.type = "button";
      button.disabled = !s.compatible || store.state.snapshot?.task != null;
      button.dataset.file = s.file;
      button.addEventListener("click", () => {
        button.disabled = true;
        actions
          .load(s.file)
          .then(() => loadDialog.close())
          .catch((e: unknown) => {
            notify(errorText(e));
            button.disabled = false;
          });
      });
      const save = el(
        "td",
        {},
        el("span", { text: s.label || "(no label)" }),
        el("span", { className: "sub kind", text: `${s.kind} · generation ${s.generation}` }),
      );
      if (s.note) {
        save.append(el("span", { className: `sub ${s.compatible ? "warn" : "bad"}`, text: s.note }));
      }
      return el(
        "tr",
        {},
        el(
          "td",
          {},
          el("span", { text: s.worldName || "(unknown)" }),
          el("span", { className: "sub", text: s.worldId.slice(0, 8) }),
        ),
        save,
        el("td", { text: s.compatible ? formatSimMinute(s.simMinute) : "—" }),
        el("td", { text: formatWhen(s.createdUnixMs) }),
        el("td", { text: formatBytes(s.sizeBytes) }),
        el("td", {}, button),
      );
    });
    loadBody.replaceChildren(
      el("table", { className: "saves" }, el("thead", {}, head), el("tbody", {}, ...rows)),
    );
  };
  const refreshSaves = () => {
    loadBody.replaceChildren(el("p", { className: "empty", text: "Reading saves…" }));
    actions
      .listSaves()
      .then(renderSaves)
      .catch((e: unknown) => {
        const retry = el("button", { text: "Try again" });
        retry.type = "button";
        retry.addEventListener("click", refreshSaves);
        loadBody.replaceChildren(
          el("p", { className: "form-error", text: `The saves could not be read: ${errorText(e)}` }),
          retry,
        );
      });
  };
  const openLoad = () => {
    loadDialog.showModal();
    refreshSaves();
  };
  $("load-refresh").addEventListener("click", refreshSaves);
  $("load-close").addEventListener("click", () => loadDialog.close());

  // ---- Recovery ---------------------------------------------------------------------------
  const recoveryDialog = $<HTMLDialogElement>("recovery-dialog");
  const recoveryError = $("recovery-error");
  let recoveryShownFor = "";
  const answerRecovery = (accept: boolean) => {
    recoveryError.hidden = true;
    actions
      .recover(accept)
      .then(() => recoveryDialog.close())
      .catch((e: unknown) => {
        recoveryError.textContent = errorText(e);
        recoveryError.hidden = false;
      });
  };
  $("recovery-accept").addEventListener("click", () => answerRecovery(true));
  $("recovery-dismiss").addEventListener("click", () => answerRecovery(false));

  // ---- Top bar, banner, task --------------------------------------------------------------
  $("btn-new").addEventListener("click", openNewWorld);
  $("btn-save").addEventListener("click", openSave);
  $("btn-load").addEventListener("click", openLoad);
  $("state-new").addEventListener("click", openNewWorld);
  $("state-load").addEventListener("click", openLoad);
  $("state-retry").addEventListener("click", () => actions.retryMap());
  const toggleRun = () => {
    const clock = store.state.snapshot?.clock;
    if (clock) void attempt(() => actions.setClock(!clock.paused, clock.speed));
  };
  $("btn-run").addEventListener("click", toggleRun);
  const runAhead = $<HTMLSelectElement>("run-ahead");
  runAhead.addEventListener("change", () => {
    const minutes = Number(runAhead.value);
    runAhead.value = "";
    if (minutes > 0) void attempt(() => actions.runAhead(minutes));
  });
  const addFamily = $<HTMLButtonElement>("add-family");
  const familyCount = $<HTMLSelectElement>("family-count");
  const families = (): number => Number(familyCount.value) || 1;
  addFamily.addEventListener("click", () => {
    const on = addFamily.getAttribute("aria-pressed") !== "true";
    actions.setPlacing(on, families());
  });
  familyCount.addEventListener("change", () => {
    actions.setPlacing(addFamily.getAttribute("aria-pressed") === "true", families());
  });
  const addDeposit = $<HTMLButtonElement>("add-deposit");
  const depositGood = $<HTMLSelectElement>("deposit-good");
  const depositExposed = $<HTMLInputElement>("deposit-exposed");
  const armDeposit = (on: boolean): void => actions.setPlacingDeposit(on, depositGood.value, depositExposed.checked);
  addDeposit.addEventListener("click", () => armDeposit(addDeposit.getAttribute("aria-pressed") !== "true"));
  depositGood.addEventListener("change", () => armDeposit(addDeposit.getAttribute("aria-pressed") === "true"));
  depositExposed.addEventListener("change", () => armDeposit(addDeposit.getAttribute("aria-pressed") === "true"));
  const addAgitator = $<HTMLButtonElement>("add-agitator");
  const agitatorIdeology = $<HTMLSelectElement>("agitator-ideology");
  const armAgitator = (on: boolean): void => actions.setPlacingAgitator(on, Number(agitatorIdeology.value) || 0);
  addAgitator.addEventListener("click", () => armAgitator(addAgitator.getAttribute("aria-pressed") !== "true"));
  agitatorIdeology.addEventListener("change", () => armAgitator(addAgitator.getAttribute("aria-pressed") === "true"));
  const addWave = $<HTMLButtonElement>("add-wave");
  const waveChoices = ["wave-households", "wave-days", "wave-months"].map((id) => $<HTMLSelectElement>(id));
  const armWave = (on: boolean): void => {
    const [households, days, months] = waveChoices.map((s) => Number(s.value));
    actions.setPlacingWave(on, households || 10, days || 1, months || 0);
  };
  addWave.addEventListener("click", () => armWave(addWave.getAttribute("aria-pressed") !== "true"));
  for (const s of waveChoices) {
    s.addEventListener("change", () => armWave(addWave.getAttribute("aria-pressed") === "true"));
  }
  document.addEventListener("keydown", (event) => {
    if (event.key !== " " || isTyping(event.target)) return;
    if (document.querySelector("dialog[open]")) return;
    if (event.target instanceof HTMLButtonElement) return;
    event.preventDefault();
    toggleRun();
  });
  $("banner-close").addEventListener("click", () => {
    store.update({
      notice: null,
      dismissedError: store.state.snapshot?.lastError ?? null,
    });
  });
  const cancel = $<HTMLButtonElement>("task-cancel");
  cancel.addEventListener("click", () => {
    cancel.disabled = true;
    void attempt(() => actions.cancelTask());
  });

  // ---- Rendering --------------------------------------------------------------------------
  const speeds = $("speeds");
  let speedsFor: Welcome | null = null;
  const renderSpeeds = (state: AppState) => {
    const welcome = state.welcome;
    if (welcome && speedsFor !== welcome) {
      speedsFor = welcome;
      const button = (m: number, title: string) => {
        const b = el("button", { text: formatSpeed(m * welcome.speed1x, welcome.speed1x) });
        b.type = "button";
        b.setAttribute("role", "radio");
        b.dataset.speed = String(m * welcome.speed1x);
        b.title = title;
        b.addEventListener("click", () => {
          void attempt(() => actions.setClock(false, m * welcome.speed1x));
        });
        return b;
      };
      // Detailed speeds, then the Accelerated ones, which live a day at a time (ADR-0011).
      const detailed = welcome.speedMultipliers.map((m) =>
        button(m, m === 1 ? "1× — one in-game day per 15 minutes" : `${m}× speed`),
      );
      const accelerated = welcome.acceleratedMultipliers.map((m) =>
        button(
          m,
          m === Infinity
            ? "Max — as fast as the world lives, a day at a time"
            : `${m}× — a day at a time, the map shown at each midnight`,
        ),
      );
      const gap = accelerated.length > 0 ? [el("span", { className: "speed-gap" })] : [];
      speeds.replaceChildren(...detailed, ...gap, ...accelerated);
    }
    const clock = state.snapshot?.clock;
    const usable =
      state.connection.state === "open" && !!state.snapshot?.world && !state.snapshot.task;
    for (const button of speeds.querySelectorAll<HTMLButtonElement>("button")) {
      const speed = Number(button.dataset.speed);
      const checked = !!clock && (speed === clock.speed || Math.abs(speed - clock.speed) < 1e-3);
      button.setAttribute("aria-checked", String(checked));
      button.disabled = !usable;
    }
  };

  const renderConnection = (state: AppState) => {
    const node = $("connection");
    const c = state.connection;
    node.dataset.state = c.state;
    switch (c.state) {
      case "connecting":
        setText(node, "Connecting…");
        break;
      case "open":
        setText(
          node,
          state.welcome ? `Connected · ${state.welcome.host} ${state.welcome.version}` : "Connected",
        );
        break;
      case "waiting":
        setText(
          node,
          `No host · retrying in ${Math.max(0, Math.ceil((c.retryAt - Date.now()) / 1000))} s`,
        );
        break;
      case "refused":
        setText(node, "Incompatible host");
        break;
    }
  };

  const renderBanner = (state: AppState) => {
    const banner = $("banner");
    const hostError = state.snapshot?.lastError ?? null;
    const showHostError = hostError && hostError !== state.dismissedError;
    const text = state.notice?.text ?? (showHostError ? hostError : null);
    banner.hidden = !text;
    banner.dataset.kind = state.notice?.kind ?? "error";
    setText($("banner-text"), text ?? "");
  };

  const renderMapState = (state: AppState) => {
    const world = state.snapshot?.world;
    const task = state.snapshot?.task;
    let show = true;
    let title = "";
    let text = "";
    let showActions = false;
    let showRetry = false;
    const c = state.connection;
    if (c.state === "refused") {
      title = "This page cannot talk to the host";
      text = c.reason;
    } else if (c.state !== "open") {
      title = c.state === "waiting" ? "Cannot reach the host" : "Connecting to the host…";
      text =
        c.state === "waiting"
          ? "Start it with civ-host serve (or tools/run.sh). This page keeps trying."
          : "";
    } else if (!state.mapAvailable) {
      title = "This browser cannot draw the map";
      text = state.map.message;
    } else if (!world && task) {
      title = task.name;
      text = task.stage;
    } else if (!world) {
      title = "No world yet";
      text = "Create a world from a seed, or load a save.";
      showActions = true;
    } else if (state.map.state === "loading") {
      title = "Drawing the map…";
    } else if (state.map.state === "error") {
      title = "The map could not be drawn";
      text = state.map.message;
      showRetry = true;
    } else {
      show = false;
    }
    $("map-state").hidden = !show;
    setText($("map-state-title"), title);
    setText($("map-state-text"), text);
    $("map-state-actions").hidden = !showActions;
    $("state-retry").hidden = !showRetry;
    $("map-hud").hidden = !(world && state.map.state === "ready");
  };

  const renderWorld = (state: AppState) => {
    const body = $("world-body");
    const world: WorldInfo | null = state.snapshot?.world ?? null;
    const key = world
      ? JSON.stringify([
          world.worldId,
          world.generation,
          world.contentChanged,
          state.snapshot?.lastAutosaveUnixMs,
          Math.floor(Date.now() / 30000),
        ])
      : "none";
    if (body.dataset.key === key) return;
    body.dataset.key = key;
    if (!world) {
      body.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    const facts: [string, string][] = [
      ["Seed", world.seed.toString()],
      ["Landscape", presetName(state.welcome, world.presetId)],
      [
        "Size",
        `${world.width} × ${world.height} cells · ${formatDistance(world.width * world.cellSizeM)}`,
      ],
      ["Elevation", `${Math.round(world.minElevationM)} – ${Math.round(world.maxElevationM)} m`],
      [
        "Land · ocean",
        `${formatPercent(1 - world.oceanFraction)} · ${formatPercent(world.oceanFraction)}`,
      ],
      ["Gentle ground", `${formatPercent(world.gentleLandFraction)} of dry land`],
      [
        "Rivers",
        `${world.reaches} reaches · ${world.riverLengthKm.toFixed(1)} km · up to ${world.maxDischargeM3s.toFixed(1)} m³/s`,
      ],
      ["Lakes", String(world.lakes)],
      ["Land tenure", world.regimeName || "household fields"],
      ["Saved", world.generation ? `generation ${world.generation}` : "not yet"],
      ["Autosaved", formatAge(state.snapshot?.lastAutosaveUnixMs ?? 0)],
    ];
    const list = el("dl", { className: "facts" });
    for (const [k, v] of facts) list.append(el("dt", { text: k }), el("dd", { text: v }));
    const children: Node[] = [list];
    const regime = state.welcome?.regimes.find((r) => r.id === world.regimeId);
    if (regime && regime.rules.length > 0) {
      children.push(
        el("ul", { className: "rules" }, ...regime.rules.map((rule) => el("li", { text: rule }))),
      );
    }
    if (world.contentChanged) {
      children.push(
        el("p", {
          className: "note",
          text: "This world was saved with different content than is loaded now; it continues with the current content.",
        }),
      );
    }
    body.replaceChildren(...children);
  };

  const eventsList = $("events");
  let renderedEvents: EventItem[] | null = null;
  const renderEvents = (state: AppState) => {
    if (renderedEvents === state.events) return;
    renderedEvents = state.events;
    $("events-empty").hidden = state.events.length > 0;
    eventsList.replaceChildren(
      ...state.events
        .slice()
        .reverse()
        .map((e) => {
          const item = el(
            "li",
            {},
            el("span", {
              className: "when",
              text: e.simMinute > 0 ? formatSimMinute(e.simMinute) : formatWhen(e.unixMs),
            }),
            el("span", { className: "text", text: e.text }),
          );
          item.dataset.kind = e.kind;
          return item;
        }),
    );
  };

  // ---- People ---------------------------------------------------------------------------
  const peopleBody = $("people-body");
  let peopleKey = "";
  let renderedSelection: Selection | null | undefined;
  const activityName = (welcome: Welcome | null, index: number) =>
    index === 0xffff
      ? "an activity no longer in the content"
      : (welcome?.activities[index]?.name ?? `activity ${index}`);
  const reasonLabel = (welcome: Welcome | null, code: number) =>
    welcome?.reasons[code] ?? `reason ${code}`;
  const link = (text: string, onClick: () => void) => {
    const button = el("button", { className: "link", text });
    button.type = "button";
    button.addEventListener("click", onClick);
    return button;
  };
  const meter = (label: string, value: number, high = 0.6) => {
    const bar = el("span");
    bar.style.width = `${Math.round(Math.min(1, Math.max(0, value)) * 100)}%`;
    const box = el("div", { className: `meter${value >= high ? " high" : ""}` }, bar);
    box.setAttribute("role", "meter");
    box.setAttribute("aria-label", label);
    box.setAttribute("aria-valuemin", "0");
    box.setAttribute("aria-valuemax", "1");
    box.setAttribute("aria-valuenow", value.toFixed(2));
    return [el("dt", { text: label }), el("dd", {}, box)];
  };
  /** A utility total to one decimal, without "-0.0". */
  const total = (v: number) => (Math.abs(v) < 0.05 ? 0 : v).toFixed(1);
  const optionText = (welcome: Welcome | null, o: ScoredOption) =>
    o.target ? `${activityName(welcome, o.activity)} — ${o.target}` : activityName(welcome, o.activity);
  const whyBlock = (welcome: Welcome | null, d: Decision, now: number): Node => {
    const box = el("div", { className: "why" }, el("h4", { text: "Why" }));
    if (!d.chosen) return box;
    box.append(
      el("p", {
        className: "choice",
        text: `At ${formatClockTime(d.minute, now)} chose ${optionText(welcome, d.chosen)} (${formatPercent(d.probability)} likely).`,
      }),
    );
    const terms = el("ul", { className: "terms" });
    for (const t of d.chosen.terms) {
      terms.append(
        el(
          "li",
          {},
          el("span", { text: reasonLabel(welcome, t.reason) }),
          el("span", { className: t.points < 0 ? "neg" : "pos", text: formatPoints(t.points) }),
        ),
      );
    }
    box.append(terms);
    if (d.runnerUp) {
      box.append(
        el("p", {
          className: "aside",
          text: `Next best: ${optionText(welcome, d.runnerUp)}, ${total(d.runnerUp.total)} points against ${total(d.chosen.total)}.`,
        }),
      );
    }
    if (d.others.length > 0) {
      box.append(
        el("p", {
          className: "aside",
          text: `Also weighed: ${d.others.map((o) => `${activityName(welcome, o.activity)} ${total(o.total)}`).join(", ")}.`,
        }),
      );
    }
    if (d.excluded.length > 0) {
      box.append(
        el("p", {
          className: "aside",
          text: `Left out: ${d.excluded.map((x) => `${activityName(welcome, x.activity)} (${reasonLabel(welcome, x.reason)})`).join(", ")}.`,
        }),
      );
    }
    return box;
  };
  /** The domains standing is kept in, as the host names them (wire 1.26). */
  const domainsOf = (state: AppState): string[] =>
    state.standing?.domains ?? ["provision", "craft", "word", "counsel"];
  /** Whom a person knows best, and why; and their standing in their settlement (ADR-0014). */
  const tiesBlock = (state: AppState, p: PersonInfo): Node => {
    const domains = domainsOf(state);
    const box = el("div", { className: "ties" }, el("h4", { text: "Knows people" }));
    if (p.standing) {
      box.append(el("p", { className: "standing-line", text: `Standing: ${standingText(p.standing, domains)}.` }));
    }
    if (p.ties.length === 0) {
      box.append(el("p", { className: "empty", text: "Nobody yet beyond their household." }));
      return box;
    }
    box.append(
      el(
        "ul",
        {},
        ...p.ties.map((t) =>
          el(
            "li",
            {},
            link(t.name, () => actions.focusPerson(t.person)),
            el("span", { className: "aside", text: ` ${t.reason}` }),
            el("span", { className: "tie-facts", text: ` (${tieText(t, domains)})` }),
          ),
        ),
      ),
    );
    return box;
  };
  /** What a person holds against whom and why, and what they have heard that is still news,
   * with who told them (wire 1.34, ADR-0016). */
  const wordBlock = (p: PersonInfo): Node => {
    const box = el("div", { className: "word" }, el("h4", { text: "Grievances and news" }));
    if (p.grievances.length === 0) {
      box.append(el("p", { className: "empty", text: "Holds no grievance." }));
    } else {
      box.append(
        el(
          "ul",
          { className: "grievances" },
          ...p.grievances.map((g) =>
            el(
              "li",
              {},
              el("span", { text: `Holds a grievance ${grievanceHead(g)}` }),
              el("span", { className: "aside", text: ` (${grievanceFacts(g)})` }),
            ),
          ),
        ),
      );
    }
    if (p.heard.length === 0) {
      box.append(el("p", { className: "empty", text: "Has heard no news." }));
    } else {
      box.append(
        el(
          "ul",
          { className: "heard" },
          ...p.heard.map((h) =>
            el(
              "li",
              {},
              el("span", { text: `${claimText(h)} ` }),
              h.from !== 0
                ? el(
                    "span",
                    { className: "aside" },
                    "(told by ",
                    link(h.fromName, () => actions.focusPerson(h.from)),
                    ` ${heardWhen(h)})`,
                  )
                : el("span", { className: "aside", text: `(${heardHow(h)})` }),
            ),
          ),
        ),
      );
    }
    return box;
  };
  /** Where a person stands on the questions of the day, and what moved them (wire 1.36). */
  const opinionBlock = (p: PersonInfo): Node => {
    const box = el("div", { className: "opinions" }, el("h4", { text: "Where they stand" }));
    if (p.positions.length === 0) {
      box.append(el("p", { className: "empty", text: "Holds no position yet: one is first held on the first of a month." }));
    } else {
      box.append(
        el(
          "ul",
          {},
          ...p.positions.map((q) =>
            el(
              "li",
              {},
              el("span", { text: `On ${q.question}: ` }),
              el("span", { className: "aside", text: positionText(q) }),
            ),
          ),
        ),
      );
    }
    if (p.values.length > 0) {
      box.append(
        el(
          "p",
          { className: "values" },
          el("span", { text: "What they hold dear: " }),
          el("span", { className: "aside", text: valuesText(p.values) }),
        ),
      );
    }
    box.append(
      p.ideologies.length === 0
        ? el("p", { className: "ideologies empty", text: "Holds to no ideology." })
        : el(
            "ul",
            { className: "ideologies" },
            ...p.ideologies.map((d) =>
              el(
                "li",
                {},
                el("span", { text: `Holds to ${d.name}: ` }),
                el("span", { className: "aside", text: ideologyText(d) }),
              ),
            ),
          ),
    );
    // The faction they belong to (wire 1.40, ADR-0017 §2).
    box.append(
      p.faction
        ? el("p", { className: "faction", text: factionText(p.faction) })
        : el("p", { className: "faction empty", text: "Belongs to no faction." }),
    );
    if (p.norms.length > 0) {
      box.append(
        el(
          "ul",
          { className: "norms" },
          ...p.norms.map((n) =>
            el(
              "li",
              {},
              el("span", { text: `That ${n.statement}: ` }),
              el("span", { className: "aside", text: normText(n) }),
            ),
          ),
        ),
      );
    }
    return box;
  };
  /** The technique chosen in the inspector's introduce form, kept while the inspector is redrawn. */
  let introChoice: { person: number; technique: number } | null = null;
  let introSelect: HTMLSelectElement | null = null;
  const knowsBlock = (welcome: Welcome | null, p: PersonInfo): Node => {
    const techniques = welcome?.techniques ?? [];
    const box = el("div", { className: "knows" }, el("h4", { text: "Knows" }));
    const rows = knowRows(p.knows, techniques);
    if (rows.length === 0) {
      box.append(el("p", { className: "empty", text: p.alive ? "Nothing yet." : "Nothing." }));
    } else {
      box.append(
        el(
          "ul",
          {},
          ...rows.map((r) =>
            el(
              "li",
              { className: `know-${r.state}` },
              el("span", { className: "know-name", text: r.name }),
              el("span", { className: "aside", text: ` ${r.text}` }),
            ),
          ),
        ),
      );
    }
    introSelect = null;
    const options = p.alive ? introducible(p.knows, techniques) : [];
    if (options.length === 0) return box;
    const select = el("select");
    select.id = "introduce-technique";
    for (const o of options) {
      const option = el("option", { text: o.heard ? `${o.name} (heard of)` : o.name });
      option.value = String(o.technique);
      select.append(option);
    }
    const chosen = introChoice;
    if (chosen?.person === p.id && options.some((o) => o.technique === chosen.technique)) {
      select.value = String(chosen.technique);
    }
    select.addEventListener("change", () => {
      introChoice = { person: p.id, technique: Number(select.value) };
    });
    const label = el("label", { text: "Introduce" });
    label.htmlFor = select.id;
    const teach = el("button", { text: "Teach" });
    teach.type = "submit";
    teach.id = "introduce-teach";
    teach.title = "They come to know it at once (god tool)";
    const tell = el("button", { text: "Tell of it" });
    tell.type = "button";
    tell.id = "introduce-tell";
    tell.title = "They only hear of it (god tool)";
    const form = el("form", { className: "introduce" }, label, select, teach, tell);
    const go = (awareOnly: boolean) => {
      teach.disabled = true;
      tell.disabled = true;
      void actions.introduceTechnique(p.id, Number(select.value), awareOnly);
    };
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      go(false);
    });
    tell.addEventListener("click", () => go(true));
    introSelect = select;
    box.append(form);
    return box;
  };
  /** The observer's hand on one person (M4c slice AJ, ADR-0016 §5): what the observer has done
   * to them and what came of it, and the two god tools that reach them, a whisper of news they
   * have not heard and an ideology to hear of. Nothing here chooses for them. */
  let observerSelects: HTMLSelectElement[] = [];
  const observerBlock = (welcome: Welcome | null, p: PersonInfo): Node => {
    const box = el("div", { className: "observer-hand" }, el("h4", { text: "The observer's hand" }));
    observerSelects = [];
    if (p.influences.length === 0) {
      box.append(el("p", { className: "empty", text: "The observer has not reached them." }));
    } else {
      box.append(
        el(
          "ul",
          { className: "influences" },
          ...p.influences.map((i) =>
            el(
              "li",
              {},
              el("span", { text: `${i.what}. ` }),
              el("span", { className: "aside", text: `(${lawDayText(i.minute)}; one recorded influence)` }),
            ),
          ),
        ),
      );
    }
    if (p.news.length > 0) {
      const select = el("select");
      select.id = "whisper-claim";
      for (const n of p.news) {
        const option = el("option", { text: claimText(n) });
        option.value = String(n.claim);
        select.append(option);
      }
      const label = el("label", { text: "Whisper" });
      label.htmlFor = select.id;
      const go = el("button", { text: "Whisper" });
      go.type = "submit";
      go.id = "whisper-go";
      go.title = "They hear it, from no one; what they do with it is theirs (god tool)";
      const form = el("form", { className: "whisper" }, label, select, go);
      form.addEventListener("submit", (event) => {
        event.preventDefault();
        go.disabled = true;
        void actions.whisper(p.id, Number(select.value));
      });
      observerSelects.push(select);
      box.append(form);
    }
    const ideologies = welcome?.ideologies ?? [];
    if (ideologies.length > 0) {
      const select = el("select");
      select.id = "tell-ideology";
      ideologies.forEach((d, k) => {
        const option = el("option", { text: d.name });
        option.value = String(k);
        option.title = d.legitimacy;
        select.append(option);
      });
      const label = el("label", { text: "Tell of" });
      label.htmlFor = select.id;
      const go = el("button", { text: "Tell them" });
      go.type = "submit";
      go.id = "tell-ideology-go";
      go.title = "They hear of it from no one and weigh it by what they hold dear (god tool)";
      const form = el("form", { className: "tell-ideology" }, label, select, go);
      form.addEventListener("submit", (event) => {
        event.preventDefault();
        go.disabled = true;
        void actions.tellOfIdeology(p.id, Number(select.value));
      });
      observerSelects.push(select);
      box.append(form);
    }
    // A blessing or a curse on their own luck in material things (M4c slice AJ).
    {
      const period = el("select");
      period.id = "bless-days";
      for (const [days, text] of [
        [30, "a month"],
        [90, "a season"],
        [365, "a year"],
        [1825, "five years"],
      ] as const) {
        const option = el("option", { text });
        option.value = String(days);
        period.append(option);
      }
      period.value = "365";
      const strength = el("select");
      strength.id = "bless-share";
      strength.setAttribute("aria-label", "How far it moves their draws");
      for (const [share, text] of [
        [0.1, "a little"],
        [0.25, "a quarter of the way"],
        [0.5, "half the way"],
      ] as const) {
        const option = el("option", { text });
        option.value = String(share);
        strength.append(option);
      }
      strength.value = "0.25";
      const label = el("label", { text: "For" });
      label.htmlFor = period.id;
      const blessButton = el("button", { text: "Bless" });
      blessButton.type = "submit";
      blessButton.id = "bless-go";
      blessButton.title = "Their own chances of illness or accident fall, and of finding things out rise (god tool)";
      const curseButton = el("button", { text: "Curse" });
      curseButton.type = "button";
      curseButton.id = "curse-go";
      curseButton.title = "Their own chances of illness or accident rise, and of finding things out fall (god tool)";
      const form = el("form", { className: "bless" }, label, period, strength, blessButton, curseButton);
      const go = (curse: boolean) => {
        blessButton.disabled = true;
        curseButton.disabled = true;
        void actions.bless(p.id, curse, Number(period.value), Number(strength.value));
      };
      form.addEventListener("submit", (event) => {
        event.preventDefault();
        go(false);
      });
      curseButton.addEventListener("click", () => go(true));
      observerSelects.push(period, strength);
      box.append(form);
    }
    return box;
  };
  const inspector = (state: AppState, sel: Selection): Node[] => {
    const welcome = state.welcome;
    const close = el("button", { className: "icon", text: "×" });
    close.type = "button";
    close.setAttribute("aria-label", "Close the inspector");
    close.addEventListener("click", () => actions.select(null));
    const p: PersonInfo | null = sel.info;
    if (!p) {
      return [
        el("div", { className: "person-head" }, el("h3", { text: "Person" }), close),
        sel.error
          ? el("p", { className: "form-error", text: sel.error })
          : el("p", { className: "empty", text: "Asking the host…" }),
      ];
    }
    const now = state.snapshot?.clock?.minute ?? 0;
    const nodes: Node[] = [
      el("div", { className: "person-head" }, el("h3", { text: p.name }), close),
      el("p", {
        className: "person-sub",
        text: `${p.sex === "female" ? "Female" : "Male"} · ${formatPersonAge(p.ageYears)} · ${p.origin}`,
      }),
    ];
    if (!p.alive && p.leftMinute > 0) {
      nodes.push(el("p", { text: `Left the valley ${formatSimMinute(p.leftMinute)}.` }));
    } else if (!p.alive) {
      nodes.push(
        el("p", { text: `Died ${formatSimMinute(p.diedMinute)}${p.cause ? ` of ${p.cause}` : ""}.` }),
      );
    } else {
      if (p.settlementName) {
        nodes.push(
          el(
            "p",
            { className: "person-sub" },
            "Lives at ",
            link(p.settlementName, () => actions.focusSettlement(p.settlement)),
          ),
        );
      }
      nodes.push(
        el(
          "p",
          { className: "doing" },
          el("span", { className: "dot" }),
          p.doing,
          el("span", {
            className: "since",
            text: `since ${formatClockTime(p.sinceMinute, now)} · this step ends ${formatClockTime(p.untilMinute, now)}`,
          }),
        ),
      );
      const dot = nodes[nodes.length - 1]?.firstChild as HTMLElement | null;
      if (dot) dot.style.background = cssColour(activityColour(welcome?.activities[p.activity]));
      const needs = el("dl", { className: "needs" });
      needs.append(...meter("Hunger", p.hunger, 0.5));
      needs.append(...meter("Tiredness", p.sleepPressure));
      needs.append(...meter("Loneliness", p.loneliness));
      needs.append(
        el("dt", { text: "At home" }),
        el("dd", {
          text: `food for ${formatDays(p.householdFoodDays)} (${formatDays(p.householdReadyDays)} ready to eat) · firewood for ${formatDays(p.householdFuelDays)} · water for ${formatDays(p.householdWaterDays)}`,
        }),
      );
      const goods = welcome?.goods ?? [];
      const stored = p.stores
        .filter((s) => goods[s.good]?.purpose !== "tool" && s.kg >= 0.5)
        .map((s) => `${goods[s.good]?.name ?? "?"} ${formatKg(s.kg)}`);
      if (stored.length > 0) {
        needs.append(el("dt", { text: "Stores" }), el("dd", { text: stored.join(" · ") }));
      }
      const tools = p.stores
        .filter((s) => goods[s.good]?.purpose === "tool" && s.kg >= 0.01)
        .map((s) => `${goods[s.good]?.name ?? "?"} ${formatTools(s.kg)}`);
      if (tools.length > 0) {
        needs.append(el("dt", { text: "Tools" }), el("dd", { text: tools.join(" · ") }));
      }
      const skills = [...p.skills]
        .sort((a, b) => b.level - a.level)
        .filter((k) => k.level >= 0.05)
        .map((k) => `${welcome?.skills[k.skill]?.name ?? "?"} ${formatPercent(k.level)}`);
      if (skills.length > 0) {
        needs.append(el("dt", { text: "Skills" }), el("dd", { text: skills.join(" · ") }));
      }
      if (p.householdTaste) {
        needs.append(el("dt", { text: "Builds" }), el("dd", { text: p.householdTaste }));
      }
      const load = goods[p.carryGood];
      const carrying = [
        load && p.carryKg > 0
          ? `${formatKg(p.carryKg)} of ${load.name.toLowerCase()}${p.carryFoodKcal > 0 ? ` (${Math.round(p.carryFoodKcal).toLocaleString()} kcal)` : ""}`
          : "",
        p.carryWaterL > 0 ? `${p.carryWaterL.toFixed(0)} L of water` : "",
      ].filter(Boolean);
      if (carrying.length > 0) {
        needs.append(el("dt", { text: "Carrying" }), el("dd", { text: carrying.join(" and ") }));
      }
      nodes.push(needs);
      const [latest, ...earlier] = p.decisions;
      if (latest) {
        const why = whyBlock(welcome, latest, now);
        if (earlier.length > 0) {
          why.appendChild(
            el(
              "ol",
              { className: "earlier" },
              ...earlier.map((d) =>
                el("li", {
                  text: `${formatClockTime(d.minute, now)} · ${d.chosen ? activityName(welcome, d.chosen.activity) : "?"} (${formatPercent(d.probability)})`,
                }),
              ),
            ),
          );
        }
        nodes.push(why);
      } else {
        nodes.push(el("p", { className: "empty", text: "No decisions recorded yet." }));
      }
    }
    nodes.push(knowsBlock(welcome, p));
    if (p.residence.length > 0) {
      nodes.push(
        el(
          "div",
          { className: "residence" },
          el("h4", { text: p.residence.length > 1 ? "Where they have lived" : "Where they live" }),
          el("ul", {}, ...p.residence.map((line) => el("li", { text: line }))),
        ),
      );
    }
    if (p.alive && p.places.length > 0) {
      nodes.push(
        el(
          "div",
          { className: "places" },
          el("h4", { text: "Places their household knows" }),
          el("ul", {}, ...p.places.map((line) => el("li", { text: line }))),
        ),
      );
    }
    // What their household believes other settlements' sellers offer (wire 1.54, ADR-0019 §1).
    if (p.alive && p.reports.length > 0) {
      nodes.push(
        el(
          "div",
          { className: "reports" },
          el("h4", { text: "Prices their household has heard of elsewhere" }),
          el("ul", {}, ...p.reports.map((line) => el("li", { text: `${line}.` }))),
        ),
      );
    }
    // The errand their household means to run (wire 1.55, ADR-0019 §6).
    if (p.alive && p.errand) {
      nodes.push(el("p", { className: "errand", text: `${p.errand}.` }));
    }
    // The buildings of other settlements they have seen (wire 1.56, M5b slice AR).
    if (p.alive && p.seenAway) {
      nodes.push(el("p", { className: "seen-away", text: `Seen elsewhere: ${p.seenAway}.` }));
    }
    if (p.alive) nodes.push(tiesBlock(state, p), wordBlock(p), opinionBlock(p));
    if (p.alive) nodes.push(observerBlock(welcome, p));
    if (p.kin.length > 0 || p.family.length > 0) {
      nodes.push(
        el(
          "div",
          { className: "kin" },
          el("h4", { text: "Family" }),
          ...p.family.map((line) => el("p", { className: "family-note", text: line })),
          el(
            "ul",
            {},
            ...p.kin.map((k) =>
              el(
                "li",
                {},
                link(k.name, () => actions.focusPerson(k.id)),
                ` ${k.relation}${k.alive ? "" : k.left ? " (left)" : " (dead)"}`,
              ),
            ),
          ),
        ),
      );
    }
    return nodes;
  };
  const renderPeople = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const sel = state.selected;
    if (world && sel) {
      if (renderedSelection === sel) return;
      // Not while a technique is being chosen: the list would be replaced under the pointer.
      if (introSelect?.isConnected && document.activeElement === introSelect) return;
      if (observerSelects.some((x) => x.isConnected && document.activeElement === x)) return;
      renderedSelection = sel;
      peopleKey = "";
      peopleBody.replaceChildren(...inspector(state, sel));
      return;
    }
    renderedSelection = undefined;
    const people = state.snapshot?.people ?? [];
    const settlements = state.snapshot?.settlements ?? [];
    const key = JSON.stringify([
      world?.worldId ?? null,
      people.length,
      settlements.map((s) => [
        s.id,
        s.population,
        Math.floor(s.foodDays),
        s.foodShort,
        Math.round(s.harvestKg / 100),
        s.year,
        s.abandonedMinute,
        s.contacts,
        s.coalitions,
        s.style,
      ]),
      state.welcome?.activities.length ?? 0,
    ]);
    if (key === peopleKey) return;
    peopleKey = key;
    if (!world) {
      peopleBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    if (people.length === 0) {
      peopleBody.replaceChildren(
        el("p", {
          className: "empty",
          text: "Nobody lives here. Worlds saved before people existed have none; a new world begins with a founding band.",
        }),
      );
      return;
    }
    const nodes: Node[] = settlements.map((s) =>
      el(
        "p",
        {},
        link(s.name, () => actions.focusSettlement(s.id)),
        ` · ${s.population} people · food for ${formatDays(s.foodDays)}`,
        s.harvestKg > 0 ? ` · harvest so far ${formatKg(s.harvestKg)}` : "",
        s.foodShort && s.abandonedMinute < 0
          ? el("span", { className: "badge warn", text: "short of food" })
          : "",
        s.abandonedMinute >= 0
          ? el("span", {
              className: "badge",
              text: `abandoned ${formatSimMinute(s.abandonedMinute)}`,
            })
          : "",
        el("span", {
          className: "since",
          text: `founded ${formatSimMinute(s.foundedMinute)}${s.founding ? `, ${s.founding}` : ""}`,
        }),
        s.year ? el("span", { className: "since", text: `In the past year: ${s.year}.` }) : "",
        s.contacts
          ? el("span", { className: "since contacts", text: `Between settlements, ${s.contacts}.` })
          : "",
        ...s.coalitions.map((c) => el("span", { className: "since coalition", text: `${c}.` })),
        // Its way of building on the three clocks (wire 1.56, M5b slice AR).
        s.style ? el("span", { className: "since style", text: `Building: ${s.style}.` }) : "",
      ),
    );
    nodes.push(
      el("p", {
        className: "empty",
        text: "Click a person on the map to see what they are doing, and why.",
      }),
    );
    const legend = el("ul", { className: "legend activities" });
    for (const a of state.welcome?.activities ?? []) {
      const swatch = el("span", { className: "swatch" });
      swatch.style.background = cssColour(activityColour(a));
      legend.append(el("li", {}, swatch, a.name));
    }
    nodes.push(legend);
    const fieldLegend = el("ul", { className: "legend fields" });
    for (const f of FIELD_LEGEND) {
      const swatch = el("span", { className: "swatch square" });
      swatch.style.background = cssColour(f.fill);
      fieldLegend.append(el("li", {}, swatch, f.label));
    }
    nodes.push(el("p", { className: "legend-title", text: "Fields" }), fieldLegend);
    const hutLegend = el("ul", { className: "legend huts" });
    for (const h of BUILDING_LEGEND) {
      const swatch = el("span", { className: "swatch" });
      swatch.style.background = cssColour(h.fill);
      hutLegend.append(el("li", {}, swatch, h.label));
    }
    nodes.push(el("p", { className: "legend-title", text: "Huts" }), hutLegend);
    const pathLegend = el("ul", { className: "legend paths" });
    for (const p of PATH_LEGEND) {
      const swatch = el("span", { className: "swatch square" });
      swatch.style.background = cssColour(p.fill);
      pathLegend.append(el("li", {}, swatch, p.label));
    }
    nodes.push(el("p", { className: "legend-title", text: "Paths" }), pathLegend);
    peopleBody.replaceChildren(...nodes);
  };

  const firmsPanel = $("firms-panel");
  /** Opens a workshop's page and brings the workshops panel into view. */
  const showFirm = (id: number) => {
    actions.openFirm(id);
    firmsPanel.scrollIntoView({ block: "nearest" });
  };

  const chronicleList = $("chronicle");
  let renderedChronicle: ChronicleEntry[] | null = null;
  const renderChronicle = (state: AppState) => {
    if (renderedChronicle === state.chronicle) return;
    renderedChronicle = state.chronicle;
    $("chronicle-empty").hidden = state.chronicle.length > 0;
    const atEnd =
      chronicleList.scrollTop + chronicleList.clientHeight >= chronicleList.scrollHeight - 4;
    chronicleList.replaceChildren(
      ...state.chronicle.map((e) => {
          const text = el("span", { className: "text" });
          for (const span of e.spans) {
            if (span.kind === "person") {
              text.append(link(span.text, () => actions.focusPerson(span.id)));
            } else if (span.kind === "settlement") {
              text.append(link(span.text, () => actions.focusSettlement(span.id)));
            } else if (span.kind === "firm") {
              text.append(link(span.text, () => showFirm(span.id)));
            } else {
              text.append(span.text);
            }
          }
          return el("li", {}, el("span", { className: "when", text: formatSimMinute(e.minute) }), text);
        }),
    );
    // A chronicle reads oldest first; follow new entries unless the reader scrolled back.
    if (atEnd) chronicleList.scrollTop = chronicleList.scrollHeight;
  };

  // ---- Markets --------------------------------------------------------------------------
  const marketBody = $("market-body");
  let marketKey: unknown[] = [];
  /** Trades the panel lists, newest first. */
  const TRADES_SHOWN = 8;
  const SVG = "http://www.w3.org/2000/svg";
  const priceChart = (welcome: Welcome | null, m: MarketInfo, good: number, month: number) => {
    const points = priceSeries(m.history, good, month);
    const runs = sparkline(points, 96, 20);
    if (runs.length === 0) return null;
    const svg = document.createElementNS(SVG, "svg");
    svg.setAttribute("class", "spark");
    svg.setAttribute("viewBox", "-2 -2 100 24");
    svg.setAttribute("width", "100");
    svg.setAttribute("height", "24");
    svg.setAttribute("role", "img");
    const goods = welcome?.goods ?? [];
    const label = points
      .filter((p) => p.hoursPerUnit !== null)
      .map((p) => `${monthText(p.month)}: ${worthText(goods, good, [p])}`)
      .join("; ");
    svg.setAttribute("aria-label", `What it sold for, in hours of the sellers' work: ${label}`);
    const title = document.createElementNS(SVG, "title");
    title.textContent = label;
    svg.append(title);
    for (const run of runs) {
      if (run.includes(" ")) {
        const line = document.createElementNS(SVG, "polyline");
        line.setAttribute("points", run);
        svg.append(line);
      } else {
        const [x, y] = run.split(",");
        const dot = document.createElementNS(SVG, "circle");
        dot.setAttribute("cx", x ?? "0");
        dot.setAttribute("cy", y ?? "0");
        dot.setAttribute("r", "1.6");
        svg.append(dot);
      }
    }
    return el(
      "div",
      { className: "price" },
      svg,
      el("span", { className: "since", text: worthText(goods, good, points) }),
    );
  };
  const marketBlock = (state: AppState, m: MarketInfo, month: number): Node => {
    const welcome = state.welcome;
    const goods = welcome?.goods ?? [];
    const name = (g: number) => goods[g]?.name ?? "A good no longer known";
    const head = el(
      "p",
      { className: "market-head" },
      link(m.settlementName || "A settlement", () => actions.focusSettlement(m.settlement)),
      m.money >= 0
        ? el("span", { className: "badge money", text: `money: ${name(m.money).toLowerCase()}` })
        : el("span", { className: "badge barter", text: "barter" }),
    );
    const nodes: Node[] = [head, el("p", { className: "market-summary", text: m.summary })];
    // Buyers from other settlements (wire 1.54, M5b slice AP).
    if (m.outsiders) {
      nodes.push(el("p", { className: "since market-outsiders", text: `From elsewhere: ${m.outsiders}.` }));
    }
    // Trade with each other settlement, and who is on the road to buy here today (wire 1.55).
    if (m.between.length > 0) {
      nodes.push(
        el(
          "ul",
          { className: "market-between" },
          ...m.between.map((line) => el("li", { text: `${line}.` })),
        ),
      );
    }
    if (m.onTheWay) {
      nodes.push(el("p", { className: "since market-on-the-way", text: `${m.onTheWay}.` }));
    }
    const paid = paymentsText(goods, m);
    if (paid) nodes.push(el("p", { className: "aside", text: `Paid in: ${paid}.` }));
    const lines = el("ul", { className: "market-goods" });
    for (const g of m.goods) {
      const facts: string[] = [];
      if (g.sellers > 0) {
        facts.push(`${amountText(goods[g.good], g.offered)} offered by ${sellersText(g.sellers, g.workshops)}`);
      }
      if (g.sold >= 0.05) facts.push(`${amountText(goods[g.good], g.sold)} sold lately`);
      if (g.unmet >= 0.05) {
        facts.push(`${amountText(goods[g.good], g.unmet)} wanted, none on offer`);
      }
      if (g.acceptance >= 0.005) facts.push(`${formatPercent(g.acceptance)} of what is paid`);
      const item = el(
        "li",
        {},
        el("span", { className: "good", text: name(g.good) }),
        el("span", { className: "since", text: facts.join(" · ") }),
      );
      if (g.sellers > 0) {
        item.append(el("p", { className: "terms", text: `Asking ${askingText(goods, m.offers, g.good)}` }));
      }
      if (g.lastPayment >= 0) {
        item.append(
          el("p", {
            className: "terms",
            text: `Last sold for ${priceText(goods, g.good, g.lastPayment, g.lastPrice)}`,
          }),
        );
      }
      const chart = priceChart(welcome, m, g.good, month);
      if (chart) item.append(chart);
      lines.append(item);
    }
    nodes.push(lines);
    const groups = offerGroups(goods, m.offers);
    if (groups.length > 0) {
      const list = el("ul", { className: "offers" });
      for (const o of groups) {
        const what = `: ${amountText(goods[o.good], o.units)} at ${o.terms}`;
        list.append(
          o.firm
            ? el("li", {}, link(o.householdName, () => showFirm(o.household)), what)
            : el("li", {}, `${o.householdName}${what}`),
        );
      }
      nodes.push(
        el(
          "details",
          { className: "offers-list" },
          el("summary", { text: `Every offer (${groups.length})` }),
          list,
        ),
      );
    }
    if (m.recent.length > 0) {
      const trades = el("ol", { className: "trades" });
      for (const t of m.recent.slice(0, TRADES_SHOWN)) {
        trades.append(
          el(
            "li",
            {},
            el("span", { className: "when", text: formatSimMinute(t.minute) }),
            el("span", { className: "text", text: t.text }),
            el("span", { className: `badge ${t.sale ? "money" : "barter"}`, text: t.sale ? "sale" : "barter" }),
          ),
        );
      }
      nodes.push(el("h4", { text: "Latest trades" }), trades);
    }
    nodes.push(
      el("p", {
        className: "aside",
        text: `What sold and what was wanted fade by half every ${Math.round(m.memoryDays)} days. Prices are what the sellers' terms said a unit was worth, in hours of their own work.`,
      }),
    );
    return el("div", { className: "market" }, ...nodes);
  };
  /** "2 households", "1 household and a workshop", "2 workshops". */
  const sellersText = (sellers: number, workshops: number) => {
    const households = sellers - workshops;
    const parts: string[] = [];
    if (households > 0) parts.push(`${households} household${households === 1 ? "" : "s"}`);
    if (workshops > 0) parts.push(workshops === 1 ? "a workshop" : `${workshops} workshops`);
    return parts.join(" and ");
  };
  const renderMarkets = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const clock = state.snapshot?.clock ?? null;
    const month = clock ? monthIndex(clock.year, clock.month) : 0;
    const key = [world?.worldId ?? null, state.markets, state.marketsError, month, state.welcome];
    if (key.length === marketKey.length && key.every((k, i) => k === marketKey[i])) return;
    marketKey = key;
    if (!world) {
      marketBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    if (state.markets === null) {
      marketBody.replaceChildren(
        state.marketsError
          ? el("p", { className: "form-error", text: `The markets could not be read: ${state.marketsError}` })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    if (state.markets.length === 0) {
      marketBody.replaceChildren(
        el("p", {
          className: "empty",
          text: "Nobody offers anything yet. Each household looks over what it can spare once a week.",
        }),
      );
      return;
    }
    const nodes = state.markets.map((m) => marketBlock(state, m, month));
    if (state.marketsError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.marketsError}` }));
    }
    marketBody.replaceChildren(...nodes);
  };

  // ---- Workshops ------------------------------------------------------------------------
  const firmsBody = $("firms-body");
  let firmsKey: unknown[] = [];
  const firmBadge = (open: boolean) =>
    el("span", { className: `badge ${open ? "open" : "closed"}`, text: open ? "open" : "closed" });
  const firmItem = (state: AppState, f: FirmBrief): Node => {
    const goods = state.welcome?.goods ?? [];
    const facts = [`makes ${linesText(goods, f.lines)}`, `owned by ${f.ownerName}`];
    if (f.settlementName) facts.push(f.settlementName);
    facts.push(
      f.open
        ? `since ${formatSimMinute(f.foundedMinute)}`
        : `closed ${formatSimMinute(f.closedMinute)}`,
    );
    const item = el(
      "li",
      {},
      el("p", { className: "firm-head" }, link(f.name, () => actions.openFirm(f.id)), firmBadge(f.open)),
      el("p", { className: "since", text: facts.join(" · ") }),
      el("p", { className: "record", text: f.record }),
    );
    if (f.open && f.hiringH >= 0.5) {
      item.append(
        el("p", { className: "hiring", text: `Hiring: wants ${f.hiringH.toFixed(0)} h of work.` }),
      );
    }
    if (!f.open && f.closedWhy) {
      item.append(el("p", { className: "aside", text: `Given up: ${f.closedWhy}.` }));
    }
    return item;
  };
  const firmPage = (state: AppState, page: FirmPage): Node[] => {
    const back = link("All workshops", () => actions.openFirm(null));
    back.classList.add("back");
    const info = page.info;
    if (!info) {
      return [
        back,
        page.error
          ? el("p", { className: "form-error", text: `The workshop could not be read: ${page.error}` })
          : el("p", { className: "empty", text: "Asking the host…" }),
      ];
    }
    const welcome = state.welcome;
    const goods = welcome?.goods ?? [];
    const f = info.brief;
    const nodes: Node[] = [
      back,
      el("h3", { className: "firm-head" }, f.name, firmBadge(f.open)),
      el("p", { className: "record", text: f.record }),
    ];
    const facts: [string, string][] = [
      ["Makes", linesText(goods, f.lines)],
      ["Owned by", `${f.ownerName}, since ${formatSimMinute(info.ownerSinceMinute)}`],
      [
        "Set up",
        `by ${info.founderName} on ${formatSimMinute(f.foundedMinute)}${f.settlementName ? ` in ${f.settlementName}` : ""}`,
      ],
    ];
    if (!f.open) {
      facts.push(["Closed", `${formatSimMinute(f.closedMinute)}: ${f.closedWhy || "no reason recorded"}`]);
    }
    facts.push([
      "Last sale",
      info.lastSaleMinute >= 0 ? formatSimMinute(info.lastSaleMinute) : "none yet",
    ]);
    facts.push(["Holds", holdingsText(goods, info.stores)]);
    if (info.offers.length > 0) {
      const terms = offerGroups(goods, info.offers)
        .map((o) => `${amountText(goods[o.good], o.units)} at ${o.terms}`)
        .join("; ");
      facts.push(["Asking", terms]);
    }
    if (info.wage) {
      const work = welcome?.activities[info.wage.activity]?.name.toLowerCase() ?? "its work";
      facts.push([
        "Wage",
        `${info.wage.text} The work is to ${work}; an hour's pay is worth ${info.wage.hourH.toFixed(1)} h of its owners' own work.`,
      ]);
    }
    const list = el("dl", { className: "facts" });
    for (const [k, v] of facts) list.append(el("dt", { text: k }), el("dd", { text: v }));
    nodes.push(list);
    if (info.months.length > 0) {
      nodes.push(el("p", { className: "aside", text: lifeText(info.months) }));
      const head = el(
        "tr",
        {},
        ...["Month", "Made", "Sold", "Worked", "Paid", "Costs", "Net", "Stock"].map((h) =>
          el("th", { text: h }),
        ),
      );
      const rows = statementRows(goods, f.lines, info.months).map((r) =>
        el(
          "tr",
          {},
          ...[r.label, r.made, r.sold, r.worked, r.income, r.costs, r.margin, r.stock].map((v) =>
            el("td", { text: v }),
          ),
        ),
      );
      nodes.push(
        el("h4", { text: "Monthly statements" }),
        el(
          "div",
          { className: "table-scroll" },
          el("table", { className: "statements" }, el("thead", {}, head), el("tbody", {}, ...rows)),
        ),
        el("p", {
          className: "aside",
          text: "Worked: hours of work for it, and in brackets those hired. Paid, costs (its inputs and wages), net and stock: worth in hours of its owners' own work, what the goods would have cost them to get themselves.",
        }),
      );
    }
    if (info.entries.length > 0) {
      const books = el("ol", { className: "books" });
      for (const e of info.entries.slice(0, ENTRIES_SHOWN)) {
        books.append(
          el(
            "li",
            {},
            el("span", { className: "when", text: formatSimMinute(e.minute) }),
            el("span", { className: "text", text: e.text }),
          ),
        );
      }
      nodes.push(el("h4", { text: "Latest in its books" }), books);
    }
    return nodes;
  };
  const renderFirms = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.firms, state.firmsError, state.firm, state.welcome];
    if (key.length === firmsKey.length && key.every((k, i) => k === firmsKey[i])) return;
    firmsKey = key;
    if (!world) {
      firmsBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    if (state.firm) {
      firmsBody.replaceChildren(...firmPage(state, state.firm));
      return;
    }
    if (state.firms === null) {
      firmsBody.replaceChildren(
        state.firmsError
          ? el("p", { className: "form-error", text: `The workshops could not be read: ${state.firmsError}` })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    if (state.firms.length === 0) {
      firmsBody.replaceChildren(
        el("p", {
          className: "empty",
          text: "No workshop yet. A household sets one up when it makes a tool to sell: one its neighbours want and nobody offers, or one that sold lately and nobody offers now.",
        }),
      );
      return;
    }
    const open = state.firms.filter((f) => f.open).length;
    const nodes: Node[] = [
      el("p", {
        className: "aside",
        text: `${open} open, ${state.firms.length - open} closed.`,
      }),
      el("ul", { className: "firms" }, ...state.firms.map((f) => firmItem(state, f))),
    ];
    if (state.firmsError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.firmsError}` }));
    }
    firmsBody.replaceChildren(...nodes);
  };

  // ---- Wealth ------------------------------------------------------------------------------
  const wealthBody = $("wealth-body");
  let wealthKey: unknown[] = [];
  const LINE_W = 220;
  const LINE_H = 48;
  const giniChart = (s: SettlementWealth): Node | null => {
    const lines = giniLines(s.history, LINE_W, LINE_H);
    if (!lines.goods) return null;
    const ns = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(ns, "svg");
    svg.setAttribute("viewBox", `0 0 ${LINE_W} ${LINE_H}`);
    svg.setAttribute("class", "gini-chart");
    svg.setAttribute("role", "img");
    svg.setAttribute(
      "aria-label",
      `Ginis at the end of each year from year ${s.history[0]?.year} to year ${s.history.at(-1)?.year}`,
    );
    for (const [key, points] of Object.entries(lines)) {
      const line = document.createElementNS(ns, "polyline");
      line.setAttribute("points", points);
      line.setAttribute("class", `gini-${key}`);
      svg.append(line);
    }
    return el(
      "figure",
      { className: "gini-figure" },
      svg,
      el(
        "figcaption",
        {},
        el("span", { className: "key gini-goods", text: "goods" }),
        el("span", { className: "key gini-worked", text: "land worked" }),
        el("span", { className: "key gini-floor", text: "floor area" }),
        el("span", { className: "aside", text: " · Gini at each year's end, 0 to 1" }),
      ),
    );
  };
  const wealthBlock = (s: SettlementWealth): Node => {
    const block = el("div", { className: "wealth-settlement" });
    block.append(el("h3", { text: s.name || "A settlement" }));
    if (!s.now) {
      block.append(el("p", { className: "empty", text: "Nobody lives here now." }));
    } else {
      const now = s.now;
      block.append(
        el("p", {
          className: "since",
          text: `${now.people} people in ${now.households} household${now.households === 1 ? "" : "s"}, as things stand.`,
        }),
      );
      const head = el("tr", {}, ...["Measure", "Level", "Gini"].map((h) => el("th", { text: h })));
      const rows = measureRows(now, s.households).map((r) =>
        el("tr", {}, ...[r.measure, r.level, r.gini].map((v) => el("td", { text: v }))),
      );
      block.append(
        el("table", { className: "measures" }, el("thead", {}, head), el("tbody", {}, ...rows)),
        el("p", { className: "record", text: spreadText(now) }),
      );
    }
    const chart = giniChart(s);
    if (chart) block.append(chart);
    if (s.history.length > 0) {
      const head = el(
        "tr",
        {},
        ...["Year", "Goods", "Worked", "Floor", "Top tenth", "Hold none"].map((h) =>
          el("th", { text: h }),
        ),
      );
      const rows = yearRows(s.history).map((r) =>
        el(
          "tr",
          {},
          ...[String(r.year), r.goods, r.worked, r.floor, r.topTenth, r.holdingNone].map((v) =>
            el("td", { text: v }),
          ),
        ),
      );
      block.append(
        el("h4", { text: "At each year's end" }),
        el(
          "div",
          { className: "table-scroll" },
          el("table", { className: "statements" }, el("thead", {}, head), el("tbody", {}, ...rows)),
        ),
      );
    } else {
      block.append(
        el("p", {
          className: "aside",
          text: "The measures are recorded at each year's end; none has ended here yet.",
        }),
      );
    }
    if (s.households.length > 0) {
      const head = el(
        "tr",
        {},
        ...["Household", "People", "Holds", "Works", "Goods a head", "Floor"].map((h) =>
          el("th", { text: h }),
        ),
      );
      const rows = householdRows(s.households).map((r) =>
        el(
          "tr",
          {},
          ...[r.name, r.people, r.holds, r.works, r.goods, r.floor].map((v) =>
            el("td", { text: v }),
          ),
        ),
      );
      block.append(
        el("h4", { text: "Households" }),
        el(
          "div",
          { className: "table-scroll" },
          el(
            "table",
            { className: "statements households" },
            el("thead", {}, head),
            el("tbody", {}, ...rows),
          ),
        ),
      );
      if (s.households.length > HOUSEHOLDS_SHOWN) {
        block.append(
          el("p", {
            className: "aside",
            text: `The ${HOUSEHOLDS_SHOWN} with the most goods a head of ${s.households.length}.`,
          }),
        );
      }
    }
    return block;
  };
  const renderWealth = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.wealth, state.wealthError];
    if (key.length === wealthKey.length && key.every((k, i) => k === wealthKey[i])) return;
    wealthKey = key;
    if (!world) {
      wealthBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    if (state.wealth === null) {
      wealthBody.replaceChildren(
        state.wealthError
          ? el("p", {
              className: "form-error",
              text: `The wealth measures could not be read: ${state.wealthError}`,
            })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    if (state.wealth.settlements.length === 0) {
      wealthBody.replaceChildren(
        el("p", { className: "empty", text: "Nobody has settled yet." }),
      );
      return;
    }
    const nodes: Node[] = [
      el("p", {
        className: "aside",
        text: `Under ${state.wealth.regimeName.toLowerCase()}. Goods are valued in hours of work at each settlement's prices, the median of what they cost its households to get themselves; Ginis of goods and land count each person at their household's share a head, and floor area is compared house by house.`,
      }),
      ...state.wealth.settlements.map((s) => wealthBlock(s)),
    ];
    if (state.wealthError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.wealthError}` }));
    }
    wealthBody.replaceChildren(...nodes);
  };

  // ---- Weather -----------------------------------------------------------------------------
  const weatherBody = $("weather-body");
  let weatherKey: unknown[] = [];
  const renderWeather = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.weather, state.weatherError];
    if (key.length === weatherKey.length && key.every((k, i) => k === weatherKey[i])) return;
    weatherKey = key;
    if (!world) {
      weatherBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    if (state.weather === null) {
      weatherBody.replaceChildren(
        state.weatherError
          ? el("p", {
              className: "form-error",
              text: `The weather could not be read: ${state.weatherError}`,
            })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    const w = state.weather;
    const nodes: Node[] = [];
    if (w.today) {
      nodes.push(el("p", { className: "weather-today", text: `Today: ${w.today.words}.` }));
    }
    nodes.push(
      el("p", {
        className: "aside",
        text: `On the valley floor (${Math.round(w.heightM)} m), month by month and year by year, each against what it usually brings (in brackets): about ${Math.round(w.annualMm)} mm of rain and snow a year. Soil is the water in the ground under the wild cover, as a share of what it holds.`,
      }),
    );
    const monthHead = el(
      "tr",
      {},
      ...["Month", "Rain mm", "", "Mean °C", "Frost", "Snow", "Soil"].map((h) => el("th", { text: h })),
    );
    const months = monthRows(w).map((r) =>
      el(
        "tr",
        { className: r.partial ? "partial" : "" },
        ...[r.month, r.rain, r.rainWord, r.temp, String(r.frost), String(r.snow), r.soil].map((v) =>
          el("td", { text: v }),
        ),
      ),
    );
    nodes.push(
      el(
        "div",
        { className: "table-scroll" },
        el("table", { className: "weather-months" }, el("thead", {}, monthHead), el("tbody", {}, ...months)),
      ),
    );
    const yearHead = el(
      "tr",
      {},
      ...["Year", "Rain mm", "", "Mean °C", "Wet", "Frost", "Snow"].map((h) => el("th", { text: h })),
    );
    const years = weatherYearRows(w).map((r) =>
      el(
        "tr",
        { className: r.partial ? "partial" : "" },
        ...[r.year, r.rain, r.rainWord, r.temp, String(r.wet), String(r.frost), String(r.snow)].map(
          (v) => el("td", { text: v }),
        ),
      ),
    );
    nodes.push(
      el(
        "div",
        { className: "table-scroll" },
        el("table", { className: "weather-years" }, el("thead", {}, yearHead), el("tbody", {}, ...years)),
      ),
    );
    if (state.weatherError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.weatherError}` }));
    }
    weatherBody.replaceChildren(...nodes);
  };

  // ---- Standing ----------------------------------------------------------------------------
  const standingBody = $("standing-body");
  let standingRenderKey: unknown[] = [];
  const renderStanding = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.standing, state.standingError];
    if (key.length === standingRenderKey.length && key.every((k, i) => k === standingRenderKey[i]))
      return;
    standingRenderKey = key;
    if (!world) {
      standingBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    const st = state.standing;
    if (st === null) {
      standingBody.replaceChildren(
        state.standingError
          ? el("p", {
              className: "form-error",
              text: `Standing could not be read: ${state.standingError}`,
            })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    const nodes: Node[] = [
      el("p", {
        className: "aside",
        text: "What a settlement's adults think of one another, summed from what each saw the others do: gifts, wages, rent, trades, teaching. Notables, the few counted most often among those esteemed most, consider the settlement's affairs weekly; being one grants nothing. Worked out on the first of each month.",
      }),
    ];
    if (st.minute === 0) {
      nodes.push(el("p", { className: "empty", text: "Not worked out yet: it is, on the first of the month." }));
    }
    for (const s of st.settlements) {
      const block = el("div", { className: "standing-settlement" });
      const notables = s.rows.filter((r) => r.notable);
      block.append(
        el("h3", { text: s.name || "A settlement" }),
        el("p", {
          className: "since",
          text: `${s.adults} adults; notables: ${notables.length === 0 ? "none" : notables.map((r) => r.name).join(", ")}.`,
        }),
        el(
          "ol",
          { className: "standing-rows" },
          ...s.rows.map((r) =>
            el(
              "li",
              { className: r.notable ? "notable" : "" },
              link(r.name, () => actions.focusPerson(r.person)),
              el("span", { className: "aside", text: ` ${standingText(r, st.domains)}` }),
            ),
          ),
        ),
      );
      nodes.push(block);
    }
    nodes.push(
      el("p", {
        className: "note",
        text: `${st.ties} ties kept across the world; ${st.letGo} let go for want of room.`,
      }),
    );
    if (state.standingError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.standingError}` }));
    }
    standingBody.replaceChildren(...nodes);
  };

  // ---- Government --------------------------------------------------------------------------
  const governmentBody = $("government-body");
  let governmentRenderKey: unknown[] = [];
  /** The laws opened to their histories, by id, kept across redraws. */
  const lawsOpen = new Set<number>();
  const renderGovernment = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.government, state.governmentError];
    if (
      key.length === governmentRenderKey.length &&
      key.every((k, i) => k === governmentRenderKey[i])
    )
      return;
    governmentRenderKey = key;
    if (!world) {
      governmentBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    const g = state.government;
    if (g === null) {
      governmentBody.replaceChildren(
        state.governmentError
          ? el("p", {
              className: "form-error",
              text: `The government could not be read: ${state.governmentError}`,
            })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    const nodes: Node[] = [
      el("p", {
        className: "aside",
        text: "Each settlement decides by the custom every world starts from. A law exists only if someone proposed it and a gathering passed it; each is kept with its whole history: who proposed it and why, who came and where each stood, what was paid, kept back and given.",
      }),
    ];
    if (g.polities.length === 0) {
      nodes.push(el("p", { className: "empty", text: "No settlement has a polity yet: each is founded at the first midnight." }));
    }
    for (const p of g.polities) {
      const block = el("div", { className: "polity" });
      block.append(
        el("h3", { text: p.name || "A settlement" }),
        el("p", { className: "custom", text: p.custom }),
        el("p", {
          className: "since",
          text: `${membersText(p)}; the common store holds ${p.store}.`,
        }),
      );
      // Its custom's versions, once it has been amended (wire 1.35, ADR-0017 §1).
      if (p.customHistory.length > 1) {
        block.append(
          el(
            "details",
            { className: "custom-history" },
            el("summary", { text: `The custom has changed ${p.customHistory.length - 1} time${p.customHistory.length === 2 ? "" : "s"}` }),
            el("ol", {}, ...p.customHistory.map((v) => el("li", { text: v }))),
          ),
        );
      }
      if (p.offices.length > 0) {
        block.append(
          el("ul", { className: "offices" }, ...p.offices.map((o) => el("li", { text: o }))),
        );
      }
      // Its factions (wire 1.40, ADR-0017 §2).
      if (p.factions.length > 0) {
        block.append(
          el("ul", { className: "factions" }, ...p.factions.map((v) => el("li", { text: v }))),
        );
      }
      // Its petitions (wire 1.41, ADR-0017 §3), refusals of a levy (1.42), revolts (1.43) and
      // coups (1.45).
      if (p.petitions.length > 0) {
        block.append(
          el("ul", { className: "petitions" }, ...p.petitions.map((v) => el("li", { text: v }))),
        );
      }
      if (p.refusals.length > 0) {
        block.append(
          el("ul", { className: "refusals" }, ...p.refusals.map((v) => el("li", { text: v }))),
        );
      }
      if (p.revolts.length > 0) {
        block.append(
          el("ul", { className: "revolts" }, ...p.revolts.map((v) => el("li", { text: v }))),
        );
      }
      if (p.coups.length > 0) {
        block.append(el("ul", { className: "coups" }, ...p.coups.map((v) => el("li", { text: v }))));
      }
      const label = labelText(p);
      if (label !== "") {
        block.append(
          el(
            "details",
            { className: "label" },
            el("summary", { text: `Would be called: ${label}` }),
            el("p", {
              className: "aside",
              text: "Worked out afterwards from its history over the last two years; nothing in the world reads it.",
            }),
            el("ul", {}, ...p.labelWhy.map((w) => el("li", { text: w }))),
          ),
        );
      }
      if (p.gatheringLaw !== 0 || p.gatheringCases.length > 0) {
        const hears =
          p.gatheringCases.length > 0 ? ` It is to hear ${p.gatheringCases.join("; ")}.` : "";
        block.append(
          el("p", {
            className: "gathering",
            text: `A gathering is called for ${lawDayText(p.gatheringMinute)}${p.gatheringPresent > 0 ? `; ${p.gatheringPresent} have come so far` : ""}.${hears}`,
          }),
        );
      }
      if (p.laws.length === 0) {
        block.append(el("p", { className: "empty", text: "Nothing has been proposed here." }));
      }
      for (const l of p.laws) {
        const facts = el("dl", { className: "law-history" });
        facts.append(
          el("dt", { text: "Proposed" }),
          el(
            "dd",
            {},
            link(l.sponsorName, () => actions.focusPerson(l.sponsor)),
            ` on ${lawDayText(l.proposedMinute)}, because ${l.issue}`,
          ),
        );
        if (l.decision) {
          facts.append(
            el("dt", { text: "Decided" }),
            el("dd", { text: `${lawDayText(l.decidedMinute)}: ${l.decision}` }),
          );
        }
        if (l.status === "in force" || l.status === "lapsed") {
          facts.append(el("dt", { text: "Known by" }), el("dd", { text: `${l.known} living` }));
          const levy = levyText(l);
          if (levy) facts.append(el("dt", { text: "Levy" }), el("dd", { text: levy }));
          const relief = reliefText(l);
          if (relief) facts.append(el("dt", { text: "Relief" }), el("dd", { text: relief }));
          const broken = brokenText(l);
          if (broken) facts.append(el("dt", { text: "Kept" }), el("dd", { text: broken }));
        }
        const details = el(
          "details",
          { className: `law ${l.status.replace(" ", "-")}` },
          el(
            "summary",
            {},
            el("span", { className: "law-what", text: l.what }),
            el("span", { className: "status", text: ` (${statusText(l)})` }),
          ),
          facts,
        );
        if (l.stances.length > 0) {
          details.append(
            el(
              "ol",
              { className: "stances" },
              ...l.stances.map((st) =>
                el(
                  "li",
                  { className: `stance-${st.stance}` },
                  link(st.name, () => actions.focusPerson(st.person)),
                  el("span", { className: "aside", text: ` ${stanceText(st)}` }),
                ),
              ),
            ),
          );
        }
        details.open = lawsOpen.has(l.id);
        details.addEventListener("toggle", () => {
          if (details.open) lawsOpen.add(l.id);
          else lawsOpen.delete(l.id);
        });
        block.append(details);
      }
      nodes.push(block);
    }
    if (state.governmentError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.governmentError}` }));
    }
    governmentBody.replaceChildren(...nodes);
  };

  // ---- Order (takings) ---------------------------------------------------------------------
  const orderBody = $("order-body");
  let orderRenderKey: unknown[] = [];
  const renderOrder = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.order, state.orderError];
    if (key.length === orderRenderKey.length && key.every((k, i) => k === orderRenderKey[i])) return;
    orderRenderKey = key;
    if (!world) {
      orderBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    const o = state.order;
    if (o === null) {
      orderBody.replaceChildren(
        state.orderError
          ? el("p", {
              className: "form-error",
              text: `The takings could not be read: ${state.orderError}`,
            })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    const nodes: Node[] = [
      el("p", {
        className: "aside",
        text: "Each taking is shown twice: what happened, which nobody in the world reads, and what the living believe of it and chose. They differ: a taking nobody saw is known only as a loss, and word of a taker travels only along ties.",
      }),
      el("p", { className: "order-totals", text: totalsText(o) }),
    ];
    // Watchers come to take what a refused finding owed (wire 1.46).
    if (o.encounters.length > 0) {
      nodes.push(
        el("ul", { className: "encounters" }, ...o.encounters.map((v) => el("li", { text: v }))),
      );
    }
    const known = new Map(o.known.map((k) => [k.incident, k]));
    if (o.incidents.length > 0) {
      const list = el("ol", { className: "takings" });
      for (const i of o.incidents) {
        const k = known.get(i.id);
        const choice = choiceText(k);
        list.append(
          el(
            "li",
            { className: `taking taking-${i.outcome.replace(" ", "-")}` },
            el(
              "div",
              { className: "happened" },
              el("span", { className: "layer", text: "What happened: " }),
              link(i.actorName, () => actions.focusPerson(i.actor)),
              ` ${happenedRest(i)} ${seenText(i)}.${i.watch ? ` ${i.watch}.` : ""}`,
            ),
            el(
              "div",
              { className: "believed" },
              el("span", { className: "layer", text: "What people believe: " }),
              `${believedText(k)}.${choice ? ` ${choice}.` : ""}`,
            ),
          ),
        );
        const brought = caseText(k);
        if (brought !== "") {
          list.lastElementChild?.append(
            el(
              "div",
              { className: "case" },
              el("span", { className: "layer", text: "What the gathering was told: " }),
              `${brought}.`,
            ),
          );
        }
      }
      nodes.push(list);
    }
    if (state.orderError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.orderError}` }));
    }
    orderBody.replaceChildren(...nodes);
  };

  // ---- Knowledge ---------------------------------------------------------------------------
  const knowledgeBody = $("knowledge-body");
  let knowledgeKey: unknown[] = [];
  /** The techniques opened to their details, by settlement and technique, kept across redraws. */
  const knowledgeOpen = new Set<string>();
  let knowledgeWorld: string | null = null;
  const knowledgeBlock = (s: SettlementKnowledge, techniques: TechniqueInfo[]): Node => {
    const block = el("div", { className: "knowledge-settlement" });
    block.append(
      el("h3", { text: s.name || "A settlement" }),
      el("p", { className: "since", text: summaryText(s.techniques) }),
    );
    const list = el("ul", { className: "techniques" });
    for (const r of techniqueRows(s.techniques, techniques)) {
      const key = `${s.settlement}:${r.technique}`;
      const facts = el("dl", {});
      if (r.can) facts.append(el("dt", { text: "Can" }), el("dd", { text: r.can }));
      if (r.requires) facts.append(el("dt", { text: "Needs" }), el("dd", { text: r.requires }));
      facts.append(el("dt", { text: "Known by" }), el("dd", { text: r.knowers }));
      if (r.practised) facts.append(el("dt", { text: "Practised" }), el("dd", { text: r.practised }));
      if (r.learners) facts.append(el("dt", { text: "Learning" }), el("dd", { text: r.learners }));
      if (r.heard) facts.append(el("dt", { text: "Heard of" }), el("dd", { text: r.heard }));
      if (r.trust) facts.append(el("dt", { text: "Buildings" }), el("dd", { text: r.trust }));
      const details = el(
        "details",
        { className: r.known ? "known" : "unknown" },
        el(
          "summary",
          {},
          el("span", { className: "technique-name", text: r.name }),
          el("span", { className: "status", text: r.status }),
        ),
        facts,
      );
      if (r.history.length > 0) {
        details.append(
          el("ol", { className: "history" }, ...r.history.map((h) => el("li", { text: h }))),
        );
      }
      details.open = knowledgeOpen.has(key);
      details.addEventListener("toggle", () => {
        if (details.open) knowledgeOpen.add(key);
        else knowledgeOpen.delete(key);
      });
      list.append(el("li", {}, details));
    }
    block.append(list);
    return block;
  };
  const renderKnowledge = (state: AppState) => {
    const world = state.snapshot?.world ?? null;
    const key = [world?.worldId ?? null, state.knowledge, state.knowledgeError, state.welcome];
    if (key.length === knowledgeKey.length && key.every((k, i) => k === knowledgeKey[i])) return;
    knowledgeKey = key;
    if ((world?.worldId ?? null) !== knowledgeWorld) {
      knowledgeWorld = world?.worldId ?? null;
      knowledgeOpen.clear();
    }
    if (!world) {
      knowledgeBody.replaceChildren(el("p", { className: "empty", text: "No world loaded." }));
      return;
    }
    if (state.knowledge === null) {
      knowledgeBody.replaceChildren(
        state.knowledgeError
          ? el("p", {
              className: "form-error",
              text: `What people know could not be read: ${state.knowledgeError}`,
            })
          : el("p", { className: "empty", text: "Asking the host…" }),
      );
      return;
    }
    if (state.knowledge.settlements.length === 0) {
      knowledgeBody.replaceChildren(
        el("p", { className: "empty", text: "Nobody has settled yet." }),
      );
      return;
    }
    const techniques = state.welcome?.techniques ?? [];
    const nodes: Node[] = [
      el("p", {
        className: "aside",
        text: "A settlement knows a technique while someone living there knows it. Children learn their household's work as they reach its age; others learn by working beside someone who knows it. A technique dies with the last who knew it there.",
      }),
      ...state.knowledge.settlements.map((s) => knowledgeBlock(s, techniques)),
    ];
    if (state.knowledgeError) {
      nodes.push(el("p", { className: "note", text: `Not up to date: ${state.knowledgeError}` }));
    }
    knowledgeBody.replaceChildren(...nodes);
  };

  const renderTask = (state: AppState) => {
    const task = state.snapshot?.task ?? null;
    $("task").hidden = !task;
    if (!task) {
      cancel.disabled = false;
      return;
    }
    setText($("task-name"), task.name);
    setText($("task-stage"), task.stage);
    $<HTMLProgressElement>("task-progress").value = task.fraction;
    cancel.hidden = !task.cancellable;
  };

  const renderRecovery = (state: AppState) => {
    const offer = state.snapshot?.recovery ?? null;
    if (!offer) {
      recoveryShownFor = "";
      if (recoveryDialog.open) recoveryDialog.close();
      return;
    }
    if (recoveryShownFor === offer.saveFile || recoveryDialog.open) return;
    if (document.querySelector("dialog[open]")) return;
    recoveryShownFor = offer.saveFile;
    setText($("recovery-reason"), offer.reason);
    setText($("recovery-world"), offer.worldName || "your world");
    setText($("recovery-label"), offer.saveLabel || offer.saveFile);
    setText($("recovery-time"), formatSimMinute(offer.simMinute));
    recoveryError.hidden = true;
    recoveryDialog.showModal();
  };

  const render = (state: AppState) => {
    const open = state.connection.state === "open";
    const world = state.snapshot?.world ?? null;
    const clock = state.snapshot?.clock ?? null;
    const task = state.snapshot?.task ?? null;
    setText($("world-title"), world ? world.name : "No world");
    setText(
      $("clock"),
      clock
        ? `${formatSimDate({
            year: clock.year,
            month: clock.month,
            day: clock.day,
            hour: clock.hour,
            minute: clock.minuteOfHour,
          })} · ${clock.season}${clock.weather ? ` · ${clock.weather.words}` : ""}`
        : "",
    );
    const run = $<HTMLButtonElement>("btn-run");
    run.disabled = !open || !world || !!task;
    setText(run, clock && !clock.paused ? "Pause" : "Run");
    run.setAttribute("aria-pressed", String(!!clock && !clock.paused));
    if (clock && state.welcome) {
      const daily = clock.mode === "accelerated" ? ", a day at a time" : "";
      run.title = `Speed ${formatSpeed(clock.speed, state.welcome.speed1x)}${daily} (Space toggles)`;
    }
    $<HTMLButtonElement>("btn-new").disabled = !open || !!task || !state.welcome;
    $<HTMLButtonElement>("btn-save").disabled = !open || !world;
    $<HTMLButtonElement>("btn-load").disabled = !open || !!task;
    $<HTMLSelectElement>("run-ahead").disabled = !open || !world || !!task;
    const placing = $<HTMLButtonElement>("add-family");
    placing.disabled = !open || !world;
    placing.setAttribute("aria-pressed", String(state.placing));
    placing.classList.toggle("active", state.placing);
    const families = state.placeFamilies;
    setText(placing, families === 1 ? "Add a family" : `Add ${families} families`);
    placing.title = `Send ${families === 1 ? "a family" : `${families} families together`}: click where they should settle (Esc cancels)`;
    $<HTMLSelectElement>("family-count").disabled = !open || !world;
    const deposit = $<HTMLButtonElement>("add-deposit");
    const goodSelect = $<HTMLSelectElement>("deposit-good");
    const materials = materialGoods(state.welcome?.goods ?? []);
    const optionsKey = materials.map((g) => g.id).join(",");
    if (goodSelect.dataset.goods !== optionsKey) {
      goodSelect.dataset.goods = optionsKey;
      goodSelect.replaceChildren(
        ...materials.map((g) => {
          const option = el("option", { text: g.name });
          option.value = g.id;
          return option;
        }),
      );
    }
    if (state.depositGood && goodSelect.value !== state.depositGood) goodSelect.value = state.depositGood;
    deposit.disabled = !open || !world || materials.length === 0;
    goodSelect.disabled = deposit.disabled;
    $<HTMLInputElement>("deposit-exposed").disabled = deposit.disabled;
    deposit.setAttribute("aria-pressed", String(state.placingDeposit));
    deposit.classList.toggle("active", state.placingDeposit);
    // The agitator tool (M4c slice AJ): one newcomer holding the ideology chosen.
    const agitator = $<HTMLButtonElement>("add-agitator");
    const ideaSelect = $<HTMLSelectElement>("agitator-ideology");
    const ideas = state.welcome?.ideologies ?? [];
    const ideasKey = ideas.map((d) => d.id).join(",");
    if (ideaSelect.dataset.ideas !== ideasKey) {
      ideaSelect.dataset.ideas = ideasKey;
      ideaSelect.replaceChildren(
        ...ideas.map((d, k) => {
          const option = el("option", { text: d.name });
          option.value = String(k);
          option.title = d.legitimacy;
          return option;
        }),
      );
    }
    if (ideaSelect.value !== String(state.agitatorIdeology) && ideas.length > state.agitatorIdeology) {
      ideaSelect.value = String(state.agitatorIdeology);
    }
    agitator.disabled = !open || !world || ideas.length === 0;
    ideaSelect.disabled = agitator.disabled;
    agitator.setAttribute("aria-pressed", String(state.placingAgitator));
    agitator.classList.toggle("active", state.placingAgitator);
    // The migration wave (M5a slice AN).
    const wave = $<HTMLButtonElement>("add-wave");
    wave.disabled = !open || !world;
    for (const id of ["wave-households", "wave-days", "wave-months"]) $<HTMLSelectElement>(id).disabled = wave.disabled;
    wave.setAttribute("aria-pressed", String(state.placingWave));
    wave.classList.toggle("active", state.placingWave);
    renderSpeeds(state);
    renderConnection(state);
    renderBanner(state);
    renderMapState(state);
    renderPeople(state);
    renderChronicle(state);
    renderMarkets(state);
    renderFirms(state);
    renderWealth(state);
    renderWeather(state);
    renderKnowledge(state);
    renderStanding(state);
    renderGovernment(state);
    renderOrder(state);
    renderWorld(state);
    renderEvents(state);
    renderTask(state);
    renderRecovery(state);
  };

  store.subscribe(render);
  // The connection countdown and "autosaved … ago" move with the wall clock.
  setInterval(() => render(store.state), 1000);
}
