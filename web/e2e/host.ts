// Starts and stops civ-host for end-to-end tests.

import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { existsSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..", "..");
const exe = process.platform === "win32" ? "civ-host.exe" : "civ-host";

/** The civ-host binary: TCE_HOST_BIN, else the release build, else the debug build. */
export function hostBinary(): string {
  const candidates = [
    process.env.TCE_HOST_BIN,
    path.join(repo, "kernel", "target", "release", exe),
    path.join(repo, "kernel", "target", "debug", exe),
  ].filter((p): p is string => !!p);
  const found = candidates.find((p) => existsSync(p));
  if (!found) throw new Error(`civ-host is not built; looked for ${candidates.join(", ")}`);
  return found;
}

export function tempSaves(): string {
  return mkdtempSync(path.join(tmpdir(), "tce-e2e-saves-"));
}

/**
 * Makes a world with `civ-host new` in `saves` (for example lived some days), and waits up to
 * `timeoutMs`.
 */
export function makeWorld(saves: string, args: string[], timeoutMs = 120_000): void {
  const done = spawnSync(hostBinary(), ["new", "--saves", saves, ...args], {
    cwd: repo,
    encoding: "utf8",
    timeout: timeoutMs,
  });
  if (done.status !== 0) {
    throw new Error(`civ-host new failed (${done.status}):\n${done.stdout}\n${done.stderr}`);
  }
}

/**
 * Makes a world with a workshop at work in `saves`, with the `workshop_world` example of civ-sim
 * (a short natural run may have no workshop, and runs differ), and waits. Needs cargo.
 */
export function makeWorkshopWorld(saves: string): void {
  const done = spawnSync(
    "cargo",
    ["run", "--release", "--quiet", "-p", "civ-sim", "--example", "workshop_world", "--", saves],
    { cwd: path.join(repo, "kernel"), encoding: "utf8", timeout: 600_000 },
  );
  if (done.status !== 0) {
    throw new Error(`workshop_world failed (${done.status}):\n${done.stdout}\n${done.stderr}`);
  }
}

export interface Host {
  url: string;
  process: ChildProcess;
  output: () => string;
  /** Stops the host cleanly (it saves and removes its session marker). */
  stop(): Promise<void>;
  /** Kills the host without letting it clean up, like a crash or power cut. */
  kill(): Promise<void>;
}

/** Starts civ-host on a free port serving web/dist, and waits for its URL. */
export function startHost(saves: string): Promise<Host> {
  const child = spawn(
    hostBinary(),
    ["serve", "--port", "0", "--saves", saves, "--web", path.join(repo, "web", "dist")],
    { cwd: repo, stdio: ["ignore", "pipe", "pipe"] },
  );
  let output = "";
  const exited = new Promise<void>((resolve) => child.once("exit", () => resolve()));
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`civ-host did not start:\n${output}`)), 30_000);
    const onData = (chunk: Buffer) => {
      output += chunk.toString();
      const match = output.match(/running at (http:\/\/127\.0\.0\.1:\d+\/)/);
      if (match?.[1]) {
        clearTimeout(timer);
        resolve({
          url: match[1],
          process: child,
          output: () => output,
          stop: async () => {
            child.kill("SIGTERM");
            await exited;
          },
          kill: async () => {
            child.kill("SIGKILL");
            await exited;
          },
        });
      }
    };
    child.stdout?.on("data", onData);
    child.stderr?.on("data", onData);
    child.once("exit", (code) => {
      clearTimeout(timer);
      reject(new Error(`civ-host exited with ${code}:\n${output}`));
    });
  });
}
