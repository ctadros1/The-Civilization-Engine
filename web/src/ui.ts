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
import { PATH_LEGEND } from "./paths.js";
import { HostError } from "./net/client.js";
import type {
  ChronicleEntry,
  Decision,
  EventItem,
  MarketInfo,
  PersonInfo,
  SaveEntry,
  ScoredOption,
  Welcome,
  WorldInfo,
} from "./net/messages.js";
import { activityColour, cssColour } from "./people.js";
import type { AppState, Selection, Store } from "./state.js";

export interface Actions {
  newWorld(args: {
    name: string;
    presetId: string;
    sizeCells: number;
    seed: bigint;
    bandSize: number;
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
  /** Run ahead by `minutes` of simulated time. */
  runAhead(minutes: number): Promise<void>;
  /** Arm or disarm the map tool that sends a family where the map is clicked. */
  setPlacing(on: boolean): void;
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
  const nwError = $("nw-error");
  const nwCreate = $<HTMLButtonElement>("nw-create");
  let filledFor: Welcome | null = null;

  const describePreset = () => {
    const preset = store.state.welcome?.presets.find((p) => p.id === nwPreset.value);
    setText($("nw-preset-description"), preset?.description ?? "");
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
    nwBand.min = String(welcome.bandSizeMin);
    nwBand.max = String(welcome.bandSizeMax);
    setText(
      $("nw-band-hint"),
      `People in the first band, ${welcome.bandSizeMin} to ${welcome.bandSizeMax}. They choose where to camp.`,
    );
    describePreset();
  };
  nwPreset.addEventListener("change", describePreset);
  $("nw-seed-random").addEventListener("click", () => {
    nwSeed.value = randomSeed().toString();
  });
  const openNewWorld = () => {
    const welcome = store.state.welcome;
    if (!welcome) return;
    fillNewWorld(welcome);
    nwSeed.value = randomSeed().toString();
    nwBand.value = String(welcome.bandSizeDefault);
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
    nwCreate.disabled = true;
    actions
      .newWorld({
        name: $<HTMLInputElement>("nw-name").value,
        presetId: nwPreset.value,
        sizeCells: Number(nwSize.value),
        seed: BigInt(text),
        bandSize: welcome ? band : 0,
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
  addFamily.addEventListener("click", () => {
    const on = addFamily.getAttribute("aria-pressed") !== "true";
    actions.setPlacing(on);
  });
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
      speeds.replaceChildren(
        ...welcome.speedMultipliers.map((m) => {
          const button = el("button", { text: `${m}×` });
          button.type = "button";
          button.setAttribute("role", "radio");
          button.dataset.speed = String(m * welcome.speed1x);
          button.title = m === 1 ? "1× — one in-game day per 15 minutes" : `${m}× speed`;
          button.addEventListener("click", () => {
            void attempt(() => actions.setClock(false, m * welcome.speed1x));
          });
          return button;
        }),
      );
    }
    const clock = state.snapshot?.clock;
    const usable =
      state.connection.state === "open" && !!state.snapshot?.world && !state.snapshot.task;
    for (const button of speeds.querySelectorAll<HTMLButtonElement>("button")) {
      const checked = !!clock && Math.abs(Number(button.dataset.speed) - clock.speed) < 1e-3;
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
      ["Saved", world.generation ? `generation ${world.generation}` : "not yet"],
      ["Autosaved", formatAge(state.snapshot?.lastAutosaveUnixMs ?? 0)],
    ];
    const list = el("dl", { className: "facts" });
    for (const [k, v] of facts) list.append(el("dt", { text: k }), el("dd", { text: v }));
    const children: Node[] = [list];
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
        s.foodShort ? el("span", { className: "badge warn", text: "short of food" }) : "",
        el("span", { className: "since", text: `founded ${formatSimMinute(s.foundedMinute)}` }),
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
    const paid = paymentsText(goods, m);
    if (paid) nodes.push(el("p", { className: "aside", text: `Paid in: ${paid}.` }));
    const lines = el("ul", { className: "market-goods" });
    for (const g of m.goods) {
      const facts: string[] = [];
      if (g.sellers > 0) {
        facts.push(
          `${amountText(goods[g.good], g.offered)} offered by ${g.sellers} household${g.sellers === 1 ? "" : "s"}`,
        );
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
        list.append(
          el(
            "li",
            {},
            `${o.householdName}: ${amountText(goods[o.good], o.units)} for ${o.terms}`,
          ),
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
          })} · ${clock.season}`
        : "",
    );
    const run = $<HTMLButtonElement>("btn-run");
    run.disabled = !open || !world || !!task;
    setText(run, clock && !clock.paused ? "Pause" : "Run");
    run.setAttribute("aria-pressed", String(!!clock && !clock.paused));
    if (clock && state.welcome) {
      run.title = `Speed ${formatSpeed(clock.speed, state.welcome.speed1x)} (Space toggles)`;
    }
    $<HTMLButtonElement>("btn-new").disabled = !open || !!task || !state.welcome;
    $<HTMLButtonElement>("btn-save").disabled = !open || !world;
    $<HTMLButtonElement>("btn-load").disabled = !open || !!task;
    $<HTMLSelectElement>("run-ahead").disabled = !open || !world || !!task;
    const placing = $<HTMLButtonElement>("add-family");
    placing.disabled = !open || !world;
    placing.setAttribute("aria-pressed", String(state.placing));
    placing.classList.toggle("active", state.placing);
    renderSpeeds(state);
    renderConnection(state);
    renderBanner(state);
    renderMapState(state);
    renderPeople(state);
    renderChronicle(state);
    renderMarkets(state);
    renderWorld(state);
    renderEvents(state);
    renderTask(state);
    renderRecovery(state);
  };

  store.subscribe(render);
  // The connection countdown and "autosaved … ago" move with the wall clock.
  setInterval(() => render(store.state), 1000);
}
