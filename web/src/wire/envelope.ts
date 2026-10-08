// The commons-wire frame envelope (ADR-0001): a fixed 56-byte little-endian header and an opaque
// payload. This is the only hand-written codec on the boundary; it must reproduce the golden
// vectors in commons/crates/commons-wire/tests/golden.json byte for byte.

export const MAGIC = [0x43, 0x57, 0x49, 0x52]; // "CWIR"
export const ENVELOPE_VERSION = 1;
export const HEADER_LEN = 56;
export const FLAG_PAYLOAD_CRC32 = 0x0001;
export const DEFAULT_MAX_PAYLOAD_LEN = 64 * 1024 * 1024;

export enum FrameKind {
  Hello = 1,
  Welcome = 2,
  Snapshot = 3,
  Delta = 4,
  Events = 5,
  Command = 6,
  Query = 7,
  Response = 8,
  Error = 9,
  Heartbeat = 10,
}

export interface SchemaId {
  /** Four tag bytes, e.g. "TCE\0". */
  tag: string;
  major: number;
  minor: number;
}

export interface FrameMeta {
  kind: FrameKind;
  schema: SchemaId;
  epoch: number;
  sequence: bigint;
  correlation: bigint;
  simTime: bigint;
}

export interface Frame {
  meta: FrameMeta;
  flags: number;
  payloadCrc32: number;
  payload: Uint8Array;
}

/** The schema the web shell speaks: TCE 1.22 (civ-schema WIRE_SCHEMA). */
export const TCE_SCHEMA: SchemaId = { tag: "TCE\0", major: 1, minor: 28 };

export class WireError extends Error {}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

/** CRC-32 (IEEE 802.3), as zlib and crc32fast compute it. */
export function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) {
    crc = (CRC_TABLE[(crc ^ bytes[i]!) & 0xff]! ^ (crc >>> 8)) >>> 0;
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function requiresCorrelation(kind: FrameKind): boolean {
  return kind === FrameKind.Command || kind === FrameKind.Query || kind === FrameKind.Response;
}

function tagBytes(tag: string): number[] {
  if (tag.length !== 4) throw new WireError(`schema tag must be 4 bytes, got ${JSON.stringify(tag)}`);
  return Array.from(tag, (c) => c.charCodeAt(0) & 0xff);
}

/** Encodes one frame. */
export function encode(
  meta: FrameMeta,
  payload: Uint8Array,
  withCrc = false,
): Uint8Array<ArrayBuffer> {
  if (requiresCorrelation(meta.kind) && meta.correlation === 0n) {
    throw new WireError(`a ${FrameKind[meta.kind]} frame needs a correlation id`);
  }
  const out = new Uint8Array(HEADER_LEN + payload.length);
  const view = new DataView(out.buffer);
  out.set(MAGIC, 0);
  out[4] = ENVELOPE_VERSION;
  out[5] = meta.kind;
  view.setUint16(6, withCrc ? FLAG_PAYLOAD_CRC32 : 0, true);
  out.set(tagBytes(meta.schema.tag), 8);
  view.setUint16(12, meta.schema.major, true);
  view.setUint16(14, meta.schema.minor, true);
  view.setUint32(16, meta.epoch, true);
  view.setUint32(20, payload.length, true);
  view.setBigUint64(24, meta.sequence, true);
  view.setBigUint64(32, meta.correlation, true);
  view.setBigInt64(40, meta.simTime, true);
  view.setUint32(48, withCrc ? crc32(payload) : 0, true);
  view.setUint32(52, 0, true);
  out.set(payload, HEADER_LEN);
  return out;
}

/** Decodes exactly one frame occupying all of `bytes` (one WebSocket message). */
export function decode(bytes: Uint8Array, maxPayloadLen = DEFAULT_MAX_PAYLOAD_LEN): Frame {
  if (bytes.length < HEADER_LEN) {
    throw new WireError(`truncated: ${bytes.length} bytes, a header needs ${HEADER_LEN}`);
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  for (let i = 0; i < 4; i++) {
    if (bytes[i] !== MAGIC[i]) throw new WireError("bad magic: not a commons-wire frame");
  }
  if (bytes[4] !== ENVELOPE_VERSION) {
    throw new WireError(`unsupported envelope version ${bytes[4]}`);
  }
  const kind = bytes[5]!;
  if (kind < FrameKind.Hello || kind > FrameKind.Heartbeat) {
    throw new WireError(`unknown frame kind ${kind}`);
  }
  const flags = view.getUint16(6, true);
  if ((flags & ~FLAG_PAYLOAD_CRC32) !== 0) throw new WireError(`unknown flags ${flags}`);
  const tag = String.fromCharCode(bytes[8]!, bytes[9]!, bytes[10]!, bytes[11]!);
  const payloadLen = view.getUint32(20, true);
  const payloadCrc32 = view.getUint32(48, true);
  if (view.getUint32(52, true) !== 0) throw new WireError("reserved header word is not zero");
  if ((flags & FLAG_PAYLOAD_CRC32) === 0 && payloadCrc32 !== 0) {
    throw new WireError("a checksum is present without its flag");
  }
  if (payloadLen > maxPayloadLen) {
    throw new WireError(`payload of ${payloadLen} bytes exceeds the limit of ${maxPayloadLen}`);
  }
  const meta: FrameMeta = {
    kind: kind as FrameKind,
    schema: { tag, major: view.getUint16(12, true), minor: view.getUint16(14, true) },
    epoch: view.getUint32(16, true),
    sequence: view.getBigUint64(24, true),
    correlation: view.getBigUint64(32, true),
    simTime: view.getBigInt64(40, true),
  };
  if (requiresCorrelation(meta.kind) && meta.correlation === 0n) {
    throw new WireError(`a ${FrameKind[meta.kind]} frame without a correlation id`);
  }
  if (bytes.length !== HEADER_LEN + payloadLen) {
    throw new WireError(
      `length mismatch: the header declares ${HEADER_LEN + payloadLen} bytes, ${bytes.length} arrived`,
    );
  }
  const payload = bytes.subarray(HEADER_LEN);
  if (flags & FLAG_PAYLOAD_CRC32) {
    const computed = crc32(payload);
    if (computed !== payloadCrc32) {
      throw new WireError(`checksum mismatch: expected ${payloadCrc32}, computed ${computed}`);
    }
  }
  return { meta, flags, payloadCrc32, payload };
}

/** Whether a peer's schema can be talked to: same tag and major. */
export function compatible(a: SchemaId, b: SchemaId): boolean {
  return a.tag === b.tag && a.major === b.major;
}
