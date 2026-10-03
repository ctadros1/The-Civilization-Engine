// The shell's panels and dialogs. Every panel has empty, loading and error states (plan §7).
// Rendering is plain DOM: text and visibility are set from the store on every change.

import {
  formatAge,
  formatBytes,
  formatDistance,
  formatMapSize,
  formatPercent,
  formatSimDate,
  formatSimMinute,
  formatSpeed,
  formatWhen,
} from "./format.js";
import { HostError } from "./net/client.js";
import type { EventItem, SaveEntry, Welcome, WorldInfo } from "./net/messages.js";
import type { AppState, Store } from "./state.js";

export interface Actions {
  newWorld(args: { name: string; presetId: string; sizeCells: number; seed: bigint }): Promise<void>;
  save(label: string): Promise<void>;
  listSaves(): Promise<SaveEntry[]>;
  load(file: string): Promise<void>;
  setClock(paused: boolean, speed: number): Promise<void>;
  cancelTask(): Promise<void>;
  recover(accept: boolean): Promise<void>;
  retryMap(): void;
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
    nwCreate.disabled = true;
    actions
      .newWorld({
        name: $<HTMLInputElement>("nw-name").value,
        presetId: nwPreset.value,
        sizeCells: Number(nwSize.value),
        seed: BigInt(text),
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
    renderSpeeds(state);
    renderConnection(state);
    renderBanner(state);
    renderMapState(state);
    renderWorld(state);
    renderEvents(state);
    renderTask(state);
    renderRecovery(state);
  };

  store.subscribe(render);
  // The connection countdown and "autosaved … ago" move with the wall clock.
  setInterval(() => render(store.state), 1000);
}
