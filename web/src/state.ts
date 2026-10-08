// The shell's state: what the host last said, plus what the page itself knows. One store, plain
// objects, listeners called on every change.

import type { MapStatus } from "./map/view.js";
import type { ConnectionStatus } from "./net/client.js";
import type {
  ChronicleEntry,
  EventItem,
  FirmBrief,
  FirmInfo,
  GovernmentInfo,
  OrderInfo,
  KnowledgeInfo,
  MarketInfo,
  PersonInfo,
  Snapshot,
  StandingInfo,
  WealthInfo,
  WeatherReport,
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

/** The workshop whose page is open, and what the host last said about it. */
export interface FirmPage {
  id: number;
  info: FirmInfo | null;
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
  /** The map tool that sends a family where the map is clicked is armed. */
  placing: boolean;
  /** How many families the map tool sends together (1 to 20). */
  placeFamilies: number;
  /** The map tool that lays down a deposit where the map is clicked is armed (M3b slice Q). */
  placingDeposit: boolean;
  /** What that tool lays down: a good's content id ("" for the first material), and whether it
   * shows at the surface. */
  depositGood: string;
  depositExposed: boolean;
  /** The chronicle of the world on show, oldest first. */
  chronicle: ChronicleEntry[];
  /** The markets of the world on show (null = not read yet). */
  markets: MarketInfo[] | null;
  /** Why the markets could not be read. */
  marketsError: string | null;
  /** The workshops of the world on show, in brief (null = not read yet). */
  firms: FirmBrief[] | null;
  /** Why the workshops could not be read. */
  firmsError: string | null;
  /** The workshop whose page is open. */
  firm: FirmPage | null;
  /** The wealth measures of the world on show (null = not read yet). */
  wealth: WealthInfo | null;
  /** Why the wealth measures could not be read. */
  wealthError: string | null;
  /** What each settlement of the world on show knows (null = not read yet). */
  knowledge: KnowledgeInfo | null;
  /** Why the knowledge could not be read. */
  knowledgeError: string | null;
  /** Every month's weather in the world on show (null = not read yet; wire 1.24). */
  weather: WeatherReport | null;
  /** Why the weather could not be read. */
  weatherError: string | null;
  /** Every settlement's standing in the world on show (null = not read yet; wire 1.26). */
  standing: StandingInfo | null;
  /** Why the standing could not be read. */
  standingError: string | null;
  /** Every settlement's polity in the world on show (null = not read yet; wire 1.27). */
  government: GovernmentInfo | null;
  /** Why the government could not be read. */
  governmentError: string | null;
  /** Takings in the world on show (null = not read yet; wire 1.30). */
  order: OrderInfo | null;
  /** Why the takings could not be read. */
  orderError: string | null;
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
    placing: false,
    placeFamilies: 1,
    placingDeposit: false,
    depositGood: "",
    depositExposed: true,
    chronicle: [],
    markets: null,
    marketsError: null,
    firms: null,
    firmsError: null,
    firm: null,
    wealth: null,
    wealthError: null,
    knowledge: null,
    knowledgeError: null,
    weather: null,
    weatherError: null,
    standing: null,
    standingError: null,
    government: null,
    governmentError: null,
    order: null,
    orderError: null,
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
