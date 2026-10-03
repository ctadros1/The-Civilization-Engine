import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

import {
  FrameKind,
  HEADER_LEN,
  TCE_SCHEMA,
  WireError,
  crc32,
  decode,
  encode,
  type FrameMeta,
} from "../src/wire/envelope.js";

// The same vectors the Rust decoder is tested against (ADR-0001: one envelope, two languages).
interface Vector {
  name: string;
  hex: string;
  kind: number;
  flags: number;
  schema_tag_hex: string;
  major: number;
  minor: number;
  epoch: number;
  sequence: string;
  correlation: string;
  sim_time: string;
  payload_crc32: number;
  payload_hex: string;
}

const golden = JSON.parse(
  readFileSync(
    new URL("../../commons/crates/commons-wire/tests/golden.json", import.meta.url),
    "utf8",
  ),
) as { vectors: Vector[] };

const fromHex = (hex: string) =>
  new Uint8Array(hex.match(/../g)?.map((b) => parseInt(b, 16)) ?? []);
const toHex = (bytes: Uint8Array) =>
  Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");

function meta(kind: FrameKind, correlation = 0n): FrameMeta {
  return { kind, schema: TCE_SCHEMA, epoch: 2, sequence: 5n, correlation, simTime: 85_320n };
}

describe("golden vectors", () => {
  it("has vectors to check", () => {
    expect(golden.vectors.length).toBeGreaterThanOrEqual(3);
  });

  for (const v of golden.vectors) {
    it(`decodes ${v.name}`, () => {
      const frame = decode(fromHex(v.hex));
      expect(frame.meta.kind).toBe(v.kind);
      expect(frame.flags).toBe(v.flags);
      expect(toHex(new TextEncoder().encode(frame.meta.schema.tag))).toBe(v.schema_tag_hex);
      expect(frame.meta.schema.major).toBe(v.major);
      expect(frame.meta.schema.minor).toBe(v.minor);
      expect(frame.meta.epoch).toBe(v.epoch);
      expect(frame.meta.sequence).toBe(BigInt(v.sequence));
      expect(frame.meta.correlation).toBe(BigInt(v.correlation));
      expect(frame.meta.simTime).toBe(BigInt(v.sim_time));
      expect(frame.payloadCrc32).toBe(v.payload_crc32);
      expect(toHex(frame.payload)).toBe(v.payload_hex);
    });

    it(`re-encodes ${v.name} byte for byte`, () => {
      const frame = decode(fromHex(v.hex));
      const bytes = encode(frame.meta, frame.payload, (frame.flags & 1) === 1);
      expect(toHex(bytes)).toBe(v.hex);
    });
  }
});

describe("envelope", () => {
  it("computes the IEEE CRC-32", () => {
    expect(crc32(new TextEncoder().encode("hello"))).toBe(907060870);
    expect(crc32(new TextEncoder().encode("123456789"))).toBe(0xcbf43926);
  });

  it("round-trips with and without a checksum", () => {
    const payload = new Uint8Array([1, 2, 3, 250]);
    for (const withCrc of [false, true]) {
      const frame = decode(encode(meta(FrameKind.Command, 77n), payload, withCrc));
      expect(frame.meta).toEqual(meta(FrameKind.Command, 77n));
      expect(Array.from(frame.payload)).toEqual([1, 2, 3, 250]);
    }
  });

  it("refuses requests without a correlation id", () => {
    expect(() => encode(meta(FrameKind.Query), new Uint8Array())).toThrow(WireError);
    const bytes = encode(meta(FrameKind.Query, 1n), new Uint8Array());
    bytes.fill(0, 32, 40);
    expect(() => decode(bytes)).toThrow(/correlation/);
  });

  it("refuses malformed frames", () => {
    const good = encode(meta(FrameKind.Snapshot), new Uint8Array([9, 9]), true);
    const mutate = (f: (b: Uint8Array) => void) => {
      const b = good.slice();
      f(b);
      return b;
    };
    expect(() => decode(good.subarray(0, HEADER_LEN - 1))).toThrow(/truncated/);
    expect(() => decode(mutate((b) => (b[0] = 0)))).toThrow(/magic/);
    expect(() => decode(mutate((b) => (b[4] = 2)))).toThrow(/version/);
    expect(() => decode(mutate((b) => (b[5] = 42)))).toThrow(/kind/);
    expect(() => decode(mutate((b) => (b[6] = 0x80)))).toThrow(/flags/);
    expect(() => decode(mutate((b) => (b[52] = 1)))).toThrow(/reserved/);
    expect(() => decode(mutate((b) => (b[HEADER_LEN] = 8)))).toThrow(/checksum/);
    expect(() => decode(good.subarray(0, good.length - 1))).toThrow(/length/);
    expect(() => decode(good, 1)).toThrow(/limit/);
    const noFlag = mutate((b) => (b[6] = 0));
    expect(() => decode(noFlag)).toThrow(/without its flag/);
  });
});
