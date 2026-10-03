// The shell's state: what the host last said, plus what the page itself knows. One store, plain
// objects, listeners called on every change.

import type { MapStatus } from "./map/view.js";
import type { ConnectionStatus } from "./net/client.js";
import type {
  ChronicleEntry,
  EventItem,
  PersonInfo,
  Snapshot,
  Welcome,
} from "./net/messages.js";

export interface Notice {
  kind: "error" | "info";
  text: string;
}

/** The person the inspector shows, and what the host last said about them. */
export interface Selection {
  id: number;
  info: PersonInfo | null;
  error: string | null;
}

export interface AppState {
  connection: ConnectionStatus;
  welcome: Welcome | null;
  snapshot: Snapshot | null;
  /** World epoch of the latest snapshot (ADR-0001). */
  epoch: number;
  /** Oldest first. */
  events: EventItem[];
  map: MapStatus;
  /** Whether the browser can draw the map at all. */
  mapAvailable: boolean;
  /** A message from the page itself, such as a failed command. */
  notice: Notice | null;
  /** The host error the player dismissed. */
  dismissedError: string | null;
  /** The person being inspected. */
  selected: Selection | null;
  /** The chronicle of the world on show, oldest first. */
  chronicle: ChronicleEntry[];
}

export const MAX_EVENTS = 300;

export function initialState(): AppState {
  return {
    connection: { state: "connecting", attempt: 1 },
    welcome: null,
    snapshot: null,
    epoch: 0,
    events: [],
    map: { state: "empty", message: "" },
    mapAvailable: true,
    notice: null,
    dismissedError: null,
    selected: null,
    chronicle: [],
  };
}

/** Appends events, skipping ids already present, keeping the newest MAX_EVENTS. */
export function mergeEvents(current: EventItem[], incoming: EventItem[]): EventItem[] {
  const seen = new Set(current.map((e) => e.id));
  const merged = current.concat(incoming.filter((e) => !seen.has(e.id)));
  return merged.slice(-MAX_EVENTS);
}

export class Store {
  private listeners = new Set<(state: AppState) => void>();

  constructor(public state: AppState) {}

  update(patch: Partial<AppState>): void {
    this.state = { ...this.state, ...patch };
    for (const listener of this.listeners) listener(this.state);
  }

  subscribe(listener: (state: AppState) => void): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => this.listeners.delete(listener);
  }
}
