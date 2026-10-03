// The connection to civ-host: one WebSocket, the Hello/Welcome handshake, requests answered by
// correlation id, and the latest snapshot plus the event log (ADR-0001). Reconnects with backoff
// when the host goes away, and gives up only when the host speaks another schema major.

import {
  FrameKind,
  TCE_SCHEMA,
  compatible,
  decode,
  encode,
  type Frame,
} from "../wire/envelope.js";
import * as M from "./messages.js";

export type ConnectionStatus =
  | { state: "connecting"; attempt: number }
  | { state: "open" }
  | { state: "waiting"; retryAt: number; reason: string }
  | { state: "refused"; reason: string };

/** A request the host answered with an error frame, or that could not be answered. */
export class HostError extends Error {
  constructor(
    readonly code: M.ErrorCode | "disconnected" | "timeout",
    message: string,
  ) {
    super(message);
  }
}

export interface ClientHandlers {
  status(status: ConnectionStatus): void;
  welcome(welcome: M.Welcome): void;
  snapshot(snapshot: M.Snapshot, epoch: number): void;
  events(events: M.EventItem[]): void;
}

interface Pending {
  resolve(body: M.ResponseBody): void;
  reject(error: HostError): void;
  timer: ReturnType<typeof setTimeout>;
}

const BACKOFF_MS = [500, 1000, 2000, 4000, 8000];
const REQUEST_TIMEOUT_MS = 120_000;

export class HostClient {
  private ws: WebSocket | null = null;
  private welcomed = false;
  private refused = false;
  private stopped = false;
  private attempt = 0;
  private nextCorrelation = 1n;
  private sequences = new Map<FrameKind, bigint>();
  private pending = new Map<bigint, Pending>();
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  epoch = 0;

  constructor(
    private readonly url: string,
    private readonly on: ClientHandlers,
  ) {}

  start(): void {
    this.stopped = false;
    this.connect();
  }

  stop(): void {
    this.stopped = true;
    if (this.retryTimer) clearTimeout(this.retryTimer);
    this.ws?.close();
  }

  get connected(): boolean {
    return this.welcomed && this.ws?.readyState === WebSocket.OPEN;
  }

  private send(kind: FrameKind, correlation: bigint, payload: Uint8Array): void {
    const sequence = this.sequences.get(kind) ?? 0n;
    this.sequences.set(kind, sequence + 1n);
    const bytes = encode(
      { kind, schema: TCE_SCHEMA, epoch: this.epoch, sequence, correlation, simTime: 0n },
      payload,
    );
    this.ws?.send(bytes);
  }

  private connect(): void {
    this.attempt += 1;
    this.welcomed = false;
    this.sequences.clear();
    this.on.status({ state: "connecting", attempt: this.attempt });
    let ws: WebSocket;
    try {
      ws = new WebSocket(this.url);
    } catch (e) {
      this.scheduleRetry(`cannot open ${this.url}: ${String(e)}`);
      return;
    }
    ws.binaryType = "arraybuffer";
    this.ws = ws;
    ws.onopen = () => this.send(FrameKind.Hello, 0n, M.hello("tce-web"));
    ws.onmessage = (event) => {
      if (!(event.data instanceof ArrayBuffer)) return;
      let frame: Frame;
      try {
        frame = decode(new Uint8Array(event.data));
      } catch (e) {
        console.error("tce: dropped a malformed frame", e);
        return;
      }
      try {
        this.onFrame(frame);
      } catch (e) {
        console.error(`tce: could not handle a ${FrameKind[frame.meta.kind]} frame`, e);
      }
    };
    ws.onclose = () => {
      if (this.ws !== ws) return;
      this.ws = null;
      this.welcomed = false;
      for (const [id, p] of this.pending) {
        clearTimeout(p.timer);
        p.reject(new HostError("disconnected", "the connection to the host was lost"));
        this.pending.delete(id);
      }
      if (this.stopped || this.refused) return;
      this.scheduleRetry("the host is not reachable");
    };
  }

  private scheduleRetry(reason: string): void {
    const delay = BACKOFF_MS[Math.min(this.attempt - 1, BACKOFF_MS.length - 1)] ?? 8000;
    this.on.status({ state: "waiting", retryAt: Date.now() + delay, reason });
    this.retryTimer = setTimeout(() => this.connect(), delay);
  }

  private onFrame(frame: Frame): void {
    const { kind, correlation, epoch } = frame.meta;
    switch (kind) {
      case FrameKind.Welcome: {
        if (!compatible(frame.meta.schema, TCE_SCHEMA)) {
          this.refuse(`the host speaks schema ${frame.meta.schema.major}.x; this page speaks 1.x`);
          return;
        }
        this.welcomed = true;
        this.attempt = 0;
        this.on.status({ state: "open" });
        this.on.welcome(M.decodeWelcome(frame.payload));
        return;
      }
      case FrameKind.Snapshot:
        this.epoch = epoch;
        this.on.snapshot(M.decodeSnapshot(frame.payload), epoch);
        return;
      case FrameKind.Events:
        this.on.events(M.decodeEvents(frame.payload));
        return;
      case FrameKind.Response: {
        const pending = this.pending.get(correlation);
        if (!pending) return;
        this.pending.delete(correlation);
        clearTimeout(pending.timer);
        try {
          pending.resolve(M.decodeResponse(frame.payload));
        } catch (e) {
          pending.reject(new HostError("internal", `unreadable response: ${String(e)}`));
        }
        return;
      }
      case FrameKind.Error: {
        const error = M.decodeError(frame.payload);
        const pending = this.pending.get(correlation);
        if (pending) {
          this.pending.delete(correlation);
          clearTimeout(pending.timer);
          pending.reject(new HostError(error.code, error.message));
        } else if (!this.welcomed) {
          this.refuse(error.message);
        } else {
          console.warn(`tce: the host reported: ${error.message}`);
        }
        return;
      }
      default:
        return;
    }
  }

  private refuse(reason: string): void {
    this.refused = true;
    this.on.status({ state: "refused", reason });
    this.ws?.close();
  }

  private request(kind: FrameKind, payload: Uint8Array): Promise<M.ResponseBody> {
    if (!this.connected) {
      return Promise.reject(new HostError("disconnected", "not connected to the host"));
    }
    const correlation = this.nextCorrelation++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(correlation);
        reject(new HostError("timeout", "the host did not answer in time"));
      }, REQUEST_TIMEOUT_MS);
      this.pending.set(correlation, { resolve, reject, timer });
      this.send(kind, correlation, payload);
    });
  }

  command(payload: Uint8Array): Promise<M.ResponseBody> {
    return this.request(FrameKind.Command, payload);
  }

  query(payload: Uint8Array): Promise<M.ResponseBody> {
    return this.request(FrameKind.Query, payload);
  }

  async raster(query: M.RasterQuery): Promise<M.RasterTile> {
    const body = await this.query(M.getRaster(query));
    if (body.kind !== "raster") throw new HostError("internal", "expected a raster tile");
    return body.tile;
  }

  async hydrography(toleranceM: number): Promise<M.Hydrography> {
    const body = await this.query(M.getHydrography(toleranceM));
    if (body.kind !== "hydrography") throw new HostError("internal", "expected hydrography");
    return body.hydrography;
  }

  async saves(): Promise<M.SaveEntry[]> {
    const body = await this.query(M.listSaves());
    if (body.kind !== "saves") throw new HostError("internal", "expected a save list");
    return body.saves;
  }

  /** The routes of trips under way; ids that are not returned have ended. */
  async trips(ids: number[]): Promise<M.TripInfo[]> {
    const body = await this.query(M.getTrips(ids));
    if (body.kind !== "trips") throw new HostError("internal", "expected trips");
    return body.trips;
  }

  async person(id: number, decisions: number): Promise<M.PersonInfo> {
    const body = await this.query(M.getPerson(id, decisions));
    if (body.kind !== "person") throw new HostError("internal", "expected a person");
    return body.person;
  }

  /** Every field, with the revision it is at. */
  async fields(): Promise<{ rev: number; fields: M.FieldInfo[] }> {
    const body = await this.query(M.getFields());
    if (body.kind !== "fields") throw new HostError("internal", "expected fields");
    return { rev: body.rev, fields: body.fields };
  }

  async chronicle(
    afterSeq: number,
    limit: number,
  ): Promise<{ entries: M.ChronicleEntry[]; head: number }> {
    const body = await this.query(M.getChronicle(afterSeq, limit));
    if (body.kind !== "chronicle") throw new HostError("internal", "expected the chronicle");
    return { entries: body.entries, head: body.head };
  }
}
