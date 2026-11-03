// Who drives and who watches (wire 1.110, ADR-0039 §6), in words, and the lock a spectator's
// command controls are under. The host decides roles and refuses a spectator's commands; this
// only shows the role and keeps controls from offering what would be refused.

import type { ViewerInfo, Viewers } from "./net/messages.js";

/** Why a spectator's command controls are disabled, as their tooltip. */
export const SPECTATOR_REASON =
  "You are watching: a spectator may not command; ask the driver to hand over";

/** Every control that sends a command: time, new, save and load, recovery, the god tools, and the
 * inspector's and the people panel's tools. */
export const COMMAND_CONTROLS = [
  "#btn-run",
  "#speeds button",
  "#run-ahead",
  "#skip-ahead",
  "#skip-stop",
  "#btn-new",
  "#btn-save",
  "#btn-load",
  "#state-new",
  "#state-load",
  "#task-cancel",
  "#recovery-accept",
  "#recovery-dismiss",
  "#add-family",
  "#family-count",
  "#add-deposit",
  "#deposit-good",
  "#deposit-exposed",
  "#add-agitator",
  "#agitator-ideology",
  "#add-wave",
  "#wave-households",
  "#wave-days",
  "#wave-months",
  "#hold-rain",
  "#drought-months",
  "#drought-depth",
  "#force-climate",
  "#shake-ground",
  "#set-fire",
  "#introduce-teach",
  "#introduce-tell",
  "#whisper-go",
  "#tell-ideology-go",
  "#plague-go",
  "#bless-go",
  "#curse-go",
  "#resolution-by-focus",
  "button.pin",
].join(", ");

/** Whether this page may command: it drives, or the host says nothing of roles (before 1.110). */
export function mayCommand(viewers: Viewers | null): boolean {
  return viewers === null || viewers.role === "driver";
}

/** A connection as others see it: the name it gave, else its number, and where it is. */
export function viewerLabel(v: ViewerInfo): string {
  const name = v.name && v.name !== "tce-web" ? v.name : `viewer ${v.id}`;
  return v.local ? name : `${name} on the local network`;
}

/** This page's role in words: "You drive", "You are watching; Ada drives". Empty when the host
 * says nothing of roles. */
export function roleWords(viewers: Viewers | null): string {
  if (!viewers) return "";
  if (viewers.role === "driver") return "You drive";
  const driver = viewers.viewers.find((v) => v.id === viewers.driver);
  return driver ? `You are watching; ${viewerLabel(driver)} drives` : "You are watching; nobody drives";
}

/** Whom the driver may hand over to: every other connection. None for a spectator. */
export function handOverChoices(viewers: Viewers | null): { id: number; label: string }[] {
  if (!viewers || viewers.role !== "driver") return [];
  return viewers.viewers
    .filter((v) => v.id !== viewers.you)
    .map((v) => ({ id: v.id, label: viewerLabel(v) }));
}

/** A control the lock can disable: an HTMLButtonElement, HTMLSelectElement or HTMLInputElement. */
export interface Lockable {
  disabled: boolean;
  title: string;
  dataset: DOMStringMap;
}

/**
 * Locks `controls` for a spectator: disabled, with the reason as the tooltip (each control's own
 * tooltip kept aside). Unlocking puts the tooltip back and enables what the lock disabled; the
 * shell's own render then disables whatever it must.
 */
export function lockControls(controls: Iterable<Lockable>, locked: boolean): void {
  for (const c of controls) {
    if (locked) {
      if (c.dataset.roleLocked === undefined) {
        c.dataset.roleLocked = "1";
        c.dataset.roleTitle = c.title;
      }
      c.disabled = true;
      c.title = SPECTATOR_REASON;
    } else if (c.dataset.roleLocked !== undefined) {
      c.title = c.dataset.roleTitle ?? "";
      delete c.dataset.roleLocked;
      delete c.dataset.roleTitle;
      c.disabled = false;
    }
  }
}
