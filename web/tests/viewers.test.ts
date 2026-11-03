import { describe, expect, it } from "vitest";

import * as flatbuffers from "flatbuffers";
import * as W from "../src/schema/generated/tce/wire.js";
import { decodeError, decodeEvents, decodeEventsViewers, handOver, type Viewers } from "../src/net/messages.js";
import {
  COMMAND_CONTROLS,
  SPECTATOR_REASON,
  handOverChoices,
  lockControls,
  mayCommand,
  roleWords,
  type Lockable,
} from "../src/viewers.js";

const ada = { id: 1, name: "Ada", local: true, role: "driver" as const };
const browser = { id: 2, name: "tce-web", local: false, role: "spectator" as const };

const asDriver: Viewers = { you: 1, role: "driver", driver: 1, viewers: [ada, browser] };
const asSpectator: Viewers = { you: 2, role: "spectator", driver: 1, viewers: [ada, browser] };

function control(title: string, disabled = false): Lockable {
  return { disabled, title, dataset: {} };
}

describe("the viewer's role (wire 1.110, ADR-0039 §6)", () => {
  it("is shown in words", () => {
    expect(roleWords(asDriver)).toBe("You drive");
    expect(roleWords(asSpectator)).toBe("You are watching; Ada drives");
    expect(roleWords({ ...asSpectator, driver: 0 })).toBe("You are watching; nobody drives");
    expect(roleWords({ you: 1, role: "spectator", driver: 2, viewers: [ada, browser] })).toBe(
      "You are watching; viewer 2 on the local network drives",
    );
    // A host that says nothing of roles: nothing shown, and commands offered as before.
    expect(roleWords(null)).toBe("");
    expect(mayCommand(null)).toBe(true);
    expect(mayCommand(asDriver)).toBe(true);
    expect(mayCommand(asSpectator)).toBe(false);
  });

  it("offers the driver alone a hand-over to every other viewer", () => {
    expect(handOverChoices(asDriver)).toEqual([{ id: 2, label: "viewer 2 on the local network" }]);
    expect(handOverChoices(asSpectator)).toEqual([]);
    expect(handOverChoices(null)).toEqual([]);
  });

  it("locks a spectator's command controls with the reason, and gives them back", () => {
    const run = control("Speed 1× (Space toggles)");
    const save = control("", true);
    lockControls([run, save], true);
    expect(run.disabled).toBe(true);
    expect(run.title).toBe(SPECTATOR_REASON);
    expect(save.title).toBe(SPECTATOR_REASON);
    // Locking again keeps the controls' own tooltips aside, not the reason.
    lockControls([run, save], true);
    lockControls([run, save], false);
    expect(run.title).toBe("Speed 1× (Space toggles)");
    expect(run.disabled).toBe(false);
    expect(run.dataset.roleLocked).toBeUndefined();
    // Unlocking leaves alone what the lock never touched.
    const other = control("mine", true);
    lockControls([other], false);
    expect(other).toEqual({ disabled: true, title: "mine", dataset: {} });
  });

  it("covers the time controls, the god tools and the inspector's tools", () => {
    for (const id of ["#btn-run", "#speeds button", "#btn-new", "#btn-save", "#add-family", "#whisper-go", "#bless-go"]) {
      expect(COMMAND_CONTROLS.split(", ")).toContain(id);
    }
  });
});

describe("the viewers on the wire", () => {
  it("decode from an events frame, and the hand-over and refusal encode and decode", () => {
    const b = new flatbuffers.Builder(256);
    const name = b.createString("Ada");
    const first = W.ViewerInfo.createViewerInfo(b, 1n, name, true, W.ViewerRole.Driver);
    const second = W.ViewerInfo.createViewerInfo(b, 2n, b.createString("tce-web"), false, W.ViewerRole.Spectator);
    const list = W.Viewers.createViewersVector(b, [first, second]);
    const viewers = W.Viewers.createViewers(b, 2n, W.ViewerRole.Spectator, 1n, list);
    const none = W.Events.createEventsVector(b, []);
    W.Events.startEvents(b);
    W.Events.addEvents(b, none);
    W.Events.addViewers(b, viewers);
    b.finish(W.Events.endEvents(b));
    expect(decodeEventsViewers(b.asUint8Array())).toEqual(asSpectator);
    expect(decodeEvents(b.asUint8Array())).toEqual([]);

    const command = W.Command.getRootAsCommand(new flatbuffers.ByteBuffer(handOver(2)));
    expect(command.bodyType()).toBe(W.CommandBody.HandOver);
    expect((command.body(new W.HandOver()) as W.HandOver).to()).toBe(2n);

    const e = new flatbuffers.Builder(64);
    const message = e.createString("a spectator may not command; ask the driver to hand over");
    e.finish(W.ErrorInfo.createErrorInfo(e, W.ErrorCode.NotDriver, message));
    expect(decodeError(e.asUint8Array()).code).toBe("not-driver");
  });
});
