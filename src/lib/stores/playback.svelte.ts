import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { library, LIVE_LIBRARY } from "./library.svelte";
import { notifyError, locateMissingTrack } from "./scanner.svelte";
import { adjacentAlbum } from "../albumOrder";
import { defaultEq, matchingPreset, clampDb, EQ_PRESETS } from "../eq";
import type { PlaybackContext, Track } from "../types";

/**
 * Dual playback engine:
 *  - live (Tauri): Rust/mpv owns the album context; this store mirrors it
 *    from backend events (`playback-changed/-paused/-position/-stopped`).
 *  - fake (browser / VITE_LIB=fake): the old timer engine, so UI dev works
 *    without a backend.
 * The public API is identical either way — components never care.
 */

const TICK_MS = 250;

export type ShuffleStage = "off" | "album" | "artist" | "all";
export type RepeatStage = "off" | "album" | "track";

/** One queued/order-preview entry as mirrored from Rust (Step 7a). */
export type QueueEntryView = {
  trackId: string;
  albumId: string;
  albumIndex: number;
};

export const playback = $state({
  current: null as PlaybackContext | null,
  isPlaying: false,
  positionSec: 0,
  durationSec: 0,
  volume: 80 as number,
  /** Shuffle/repeat stages (Step 5a) — Rust owns the behavior, we mirror
   *  + persist + push. */
  shuffle: "off" as ShuffleStage,
  repeat: "off" as RepeatStage,
  /** User queue (Step 7a) — Rust owns it, we mirror it for the popover. */
  queue: [] as QueueEntryView[],
  /** First order entries after the current one ("up next from…" preview). */
  upNext: [] as QueueEntryView[],
  /** Equalizer (Step 6) — UI owns the values, Rust clamps + persists them
   *  and pushes the lavfi chain onto mpv's af. */
  eq: defaultEq(),
});

try {
  playback.volume =
    JSON.parse(localStorage.getItem("songstress.volume") ?? "80") ?? 80;
  playback.shuffle =
    JSON.parse(localStorage.getItem("songstress.shuffle") ?? '"off"') ?? "off";
  playback.repeat =
    JSON.parse(localStorage.getItem("songstress.repeat") ?? '"off"') ?? "off";
  const storedEq = JSON.parse(localStorage.getItem("songstress.eq") ?? "null");
  if (storedEq && typeof storedEq === "object") {
    playback.eq = { ...defaultEq(), ...storedEq };
  }
} catch {
  /* default */
}

// Mirror the persisted volume into SQLite right away: Rust reads it at mpv
// spawn time (first track of a session must not blare at the old default).
// Same for the shuffle/repeat stages (Rust applies them at launch).
if (LIVE_LIBRARY) {
  void invoke("set_setting", {
    key: "volume",
    value: JSON.stringify(playback.volume),
  }).catch(() => {});
  void invoke("playback_set_shuffle", { stage: playback.shuffle }).catch(() => {});
  void invoke("playback_set_repeat", { stage: playback.repeat }).catch(() => {});
}

function trackAt(albumId: string, index: number): Track | null {
  return library.tracksOf(albumId)[index] ?? null;
}

export function currentTrack(): Track | null {
  const ctx = playback.current;
  if (!ctx) return null;
  const tracks = library.tracksOf(ctx.albumId);
  // The trackId is authoritative: a rescan can re-order or re-group the
  // album mid-playback, silently shifting what the index points at.
  if (ctx.trackId) {
    const byId = tracks.find((t) => t.id === ctx.trackId);
    if (byId) return byId;
  }
  return tracks[ctx.trackIndex] ?? null;
}

// --- live engine (mpv via Rust) ----------------------------------------------

let liveWired = false;
let liveWiring: Promise<void> | null = null;

/** Idempotent; resolves once every listener is REGISTERED (not just
 * requested) so callers can't race the first backend event. */
function wireLive(): Promise<void> {
  if (liveWired || !LIVE_LIBRARY || !library.live) return Promise.resolve();
  if (liveWiring) return liveWiring;
  liveWiring = (async () => {
    await listen("playback-changed", (e) => {
      const p = e.payload as { albumId: string; trackIndex: number; trackId?: string };
      playback.current = { albumId: p.albumId, trackIndex: p.trackIndex, trackId: p.trackId };
      playback.positionSec = 0;
      playback.durationSec = currentTrack()?.durationSec ?? 0;
      playback.isPlaying = true;
    });
    await listen("playback-paused", (e) => {
      playback.isPlaying = !(e.payload as boolean);
    });
    await listen("playback-position", (e) => {
      const p = e.payload as { pos: number; dur: number };
      playback.positionSec = p.pos;
      if (p.dur > 0) playback.durationSec = p.dur;
    });
    await listen("playback-stopped", () => {
      playback.current = null;
      playback.isPlaying = false;
      playback.positionSec = 0;
      playback.durationSec = 0;
    });
    await listen("queue-changed", (e) => {
      const p = e.payload as { queue: QueueEntryView[]; upNext: QueueEntryView[] };
      playback.queue = p.queue ?? [];
      playback.upNext = p.upNext ?? [];
    });
    // Push the persisted volume into the freshly spawned engine once it has
    // had a moment to come up; harmless if it fails. Same for the persisted
    // EQ (belt and suspenders: Rust also re-applies it from settings at
    // launch, before this runs).
    const vol = playback.volume;
    const eq = { ...playback.eq, gains: [...playback.eq.gains] };
    setTimeout(() => {
      void invoke("playback_volume", { vol }).catch(() => {});
      void invoke("playback_eq", { eq }).catch(() => {});
    }, 1500);
    liveWired = true;
  })();
  return liveWiring;
}

async function livePlay(albumId: string, trackIndex: number): Promise<boolean> {
  await wireLive();
  try {
    await invoke("play_album", { albumId, trackIndex });
    // Optimistic update — don't wait for the backend event (registration
    // races / event loss should never leave the playbar stale).
    playback.current = {
      albumId,
      trackIndex,
      trackId: library.tracksOf(albumId)[trackIndex]?.id,
    };
    playback.positionSec = 0;
    playback.durationSec = trackAt(albumId, trackIndex)?.durationSec ?? 0;
    playback.isPlaying = true;
    return true;
  } catch (err) {
    console.error("play_album failed", err);
    const msg = String(err);
    const clickedId = library.tracksOf(albumId)[trackIndex]?.id;
    if (clickedId && msg.includes("missing on disk")) {
      // The file vanished without a rescan knowing: flag the row (alert
      // icon appears) and go straight into the locate flow.
      library.markMissing(clickedId);
      void locateMissingTrack(clickedId);
    } else {
      notifyError(err);
    }
    return false;
  }
}

// --- shared API ----------------------------------------------------------------

export async function playTrack(albumId: string, trackIndex: number) {
  if (!trackAt(albumId, trackIndex)) return;
  if (LIVE_LIBRARY && library.live) {
    await livePlay(albumId, trackIndex);
    return;
  }
  const track = trackAt(albumId, trackIndex)!;
  playback.current = { albumId, trackIndex };
  playback.positionSec = 0;
  playback.durationSec = track.durationSec;
  playback.isPlaying = true;
  startTimer();
}

export async function togglePlay() {
  if (!playback.current) {
    const first = library.albums[0];
    if (first) await playTrack(first.id, 0);
    return;
  }
  if (LIVE_LIBRARY && library.live) {
    await wireLive();
    try {
      const paused = await invoke<boolean>("playback_toggle");
      playback.isPlaying = !paused;
    } catch (err) {
      console.error(err);
    }
    return;
  }
  playback.isPlaying = !playback.isPlaying;
}

export async function stop() {
  if (LIVE_LIBRARY && library.live) {
    try {
      await invoke("playback_stop");
    } catch (err) {
      console.error(err);
    }
    return;
  }
  playback.isPlaying = false;
  playback.positionSec = 0;
  if (timer) clearInterval(timer);
  timer = null;
}

export async function skip(delta: number) {
  if (!playback.current) return;
  if (LIVE_LIBRARY && library.live) {
    try {
      await invoke("playback_jump", { delta });
    } catch (err) {
      console.error(err);
    }
    return;
  }
  const target = playback.current.trackIndex + delta;
  if (target >= 0 && target < library.tracksOf(playback.current.albumId).length) {
    await playTrack(playback.current.albumId, target);
  }
}

export function seekTo(sec: number) {
  const clamped = Math.min(Math.max(0, sec), playback.durationSec);
  if (LIVE_LIBRARY && library.live) {
    wireLive();
    void invoke("playback_seek", { sec: clamped }).catch((err) =>
      console.error(err),
    );
    return;
  }
  playback.positionSec = clamped;
}

let volumeWriteTimer: ReturnType<typeof setTimeout> | undefined;

export function setVolume(v: number) {
  playback.volume = Math.min(100, Math.max(0, Math.round(v)));
  localStorage.setItem("songstress.volume", JSON.stringify(playback.volume));
  // Persist to the settings table too: Rust reads it at mpv spawn time so
  // the FIRST track of a session starts at the right level.
  clearTimeout(volumeWriteTimer);
  volumeWriteTimer = setTimeout(() => {
    void invoke("set_setting", {
      key: "volume",
      value: JSON.stringify(playback.volume),
    }).catch(() => {});
  }, 300);
  if (LIVE_LIBRARY && library.live) {
    wireLive();
    void invoke("playback_volume", { vol: playback.volume }).catch(() => {});
  }
}

let preMuteVolume = 80;

/** Speaker-icon click: mute ↔ restore. */
export function toggleMute() {
  if (playback.volume > 0) {
    preMuteVolume = playback.volume;
    setVolume(0);
  } else {
    setVolume(preMuteVolume || 50);
  }
}

// --- shuffle & repeat stages (Step 5a) ----------------------------------------
// Rust owns the behavior (order building, eof handling); we mirror the stage,
// persist it (localStorage mirror + SQLite write-through), and push it.

/** Last non-off stage, so the enable checkbox restores instead of resetting.
 *  Module-private: it is a UI memory, not player state, so it is not persisted
 *  and not pushed to the menu (the menu only ever reads playback.*). */
let lastShuffle: Exclude<ShuffleStage, "off"> = "album";
let lastRepeat: Exclude<RepeatStage, "off"> = "album";

// Seeded here, not in the restore block above: `let` cells are in TDZ until
// this point in module evaluation.
if (playback.shuffle !== "off") lastShuffle = playback.shuffle;
if (playback.repeat !== "off") lastRepeat = playback.repeat;

function persistStage(key: "shuffle" | "repeat", value: string) {
  localStorage.setItem(`songstress.${key}`, JSON.stringify(value));
  void invoke("set_setting", { key, value: JSON.stringify(value) }).catch(() => {});
}

// Direct stage setters. The Global Menu CYCLES, the sidebar's segmented
// control PICKS a stage — both must go through one door so state, persistence
// and the Rust side stay in step.
export async function setShuffleStage(stage: ShuffleStage) {
  if (stage !== "off") lastShuffle = stage;
  playback.shuffle = stage;
  persistStage("shuffle", stage);
  // Rust rebuilds the queue anchored at the current track when playing.
  try {
    await invoke("playback_set_shuffle", { stage });
  } catch (err) {
    console.error("set shuffle failed", err);
  }
}

export async function setRepeatStage(stage: RepeatStage) {
  if (stage !== "off") lastRepeat = stage;
  playback.repeat = stage;
  persistStage("repeat", stage);
  try {
    await invoke("playback_set_repeat", { stage });
  } catch (err) {
    console.error("set repeat failed", err);
  }
}

/** The enable checkbox. Unchecking stores "off" but remembers the stage, so
 *  re-checking gives back what you had instead of silently picking a mode
 *  (the thing every app that forgets your filter gets dinged for). */
export function setShuffleOn(on: boolean) {
  return setShuffleStage(on ? lastShuffle : "off");
}
export function setRepeatOn(on: boolean) {
  return setRepeatStage(on ? lastRepeat : "off");
}

export async function cycleShuffle() {
  const order: ShuffleStage[] = ["off", "album", "artist", "all"];
  return setShuffleStage(order[(order.indexOf(playback.shuffle) + 1) % order.length]);
}

export async function cycleRepeat() {
  const order: RepeatStage[] = ["off", "album", "track"];
  return setRepeatStage(order[(order.indexOf(playback.repeat) + 1) % order.length]);
}

// --- equalizer (Step 6) --------------------------------------------------------
// UI owns gain values; every change is debounced to Rust, which clamps,
// persists in settings (re-applied at launch) and sets mpv's af chain.

let eqWriteTimer: ReturnType<typeof setTimeout> | undefined;

let eqHydrated = false;

/** DB wins over the localStorage mirror (same rule as the ui settings).
 *  Rust already re-applied the persisted state to mpv at launch — this only
 *  aligns the UI with it. */
export async function initEq() {
  if (eqHydrated || !LIVE_LIBRARY || !library.live) return;
  eqHydrated = true;
  try {
    const all = await invoke<Record<string, string>>("get_settings");
    const raw = all["equalizer"];
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === "object") {
        playback.eq = { ...defaultEq(), ...parsed };
      }
    }
  } catch {
    /* browser dev — localStorage keeps working */
  }
}

function pushEq() {
  localStorage.setItem("songstress.eq", JSON.stringify(playback.eq));
  clearTimeout(eqWriteTimer);
  eqWriteTimer = setTimeout(() => {
    void invoke("playback_eq", { eq: { ...playback.eq, gains: [...playback.eq.gains] } })
      .catch((err) => console.error("playback_eq failed", err));
  }, 150);
}

export function setEqEnabled(on: boolean) {
  playback.eq.enabled = on;
  pushEq();
}

export function setEqPreamp(db: number) {
  playback.eq.preampDb = clampDb(db);
  pushEq();
}

/** One band slider; divergence from every preset flips the label to Custom. */
export function setEqBand(index: number, db: number) {
  playback.eq.gains[index] = clampDb(db);
  playback.eq.preset = matchingPreset(playback.eq.gains);
  pushEq();
}

export function applyEqPreset(name: string | null) {
  const preset = EQ_PRESETS.find((p) => p.name === name);
  if (!preset) return;
  playback.eq.gains = [...preset.gains];
  playback.eq.preset = preset.name;
  pushEq();
}

/** Playback-menu cycling: Custom → first preset → … → last → Flat wraps. */
export function cycleEqPreset() {
  if (playback.eq.preset === null || playback.eq.preset === "Custom") {
    applyEqPreset(EQ_PRESETS[0].name);
    return;
  }
  const i = EQ_PRESETS.findIndex((p) => p.name === playback.eq.preset);
  applyEqPreset(EQ_PRESETS[(i + 1) % EQ_PRESETS.length].name);
}

/** The Playback pane's footer reset — its THREE subjects, not just the
 *  equalizer (a button under Repeat/Shuffle/Equalizer that only touched the
 *  third one lied about its scope, user decision 2026-08-31). Shared with
 *  the Global Menu's "Reset playback" row (2026-09-03 pass). */
export function resetPlayback() {
  setRepeatOn(false);
  setShuffleOn(false);
  setEqEnabled(false);
  // The first preset IS the flat line — named once (EQ_PRESETS[0]), the same
  // anchor cycleEqPreset wraps on, so "reset" can't drift from the table.
  applyEqPreset(EQ_PRESETS[0].name);
  setEqPreamp(0);
}

// --- user queue (Step 7a) -------------------------------------------------------
// Rust owns the queue (order engine); these just forward intent. The queue
// mirror arrives via `queue-changed`. Browser/fake dev: UI-only no-op.

export async function queueTracks(trackIds: string[], front: boolean) {
  if (!trackIds.length) return;
  if (!LIVE_LIBRARY || !library.live) return;
  try {
    await invoke("playback_queue", { trackIds, front });
  } catch (err) {
    notifyError(err);
  }
}

export async function queueRemove(pos: number) {
  if (!LIVE_LIBRARY || !library.live) return;
  try {
    await invoke("playback_queue_remove", { pos });
  } catch (err) {
    console.error("queue remove failed", err);
  }
}

export async function queueJump(pos: number) {
  if (!LIVE_LIBRARY || !library.live) return;
  try {
    await invoke("playback_queue_jump", { pos });
  } catch (err) {
    console.error("queue jump failed", err);
  }
}

export async function queueClear() {
  if (!LIVE_LIBRARY || !library.live) return;
  try {
    await invoke("playback_queue_clear");
  } catch (err) {
    console.error("queue clear failed", err);
  }
}

// --- prev/next album (Step 5b) --------------------------------------------------
// Global grid order (artist A→Z then year), wrap-around, no-op when stopped.
// Frontend-only: compute the adjacent album, play it at track 0.

export async function albumSkip(delta: 1 | -1) {
  const ctx = playback.current;
  if (!ctx) return;
  const target = adjacentAlbum(library.albums, library.artists, ctx.albumId, delta);
  if (!target) return;
  await playTrack(target.id, 0);
}

// --- fake engine (browser / fake-library dev mode) -----------------------------

let timer: ReturnType<typeof setInterval> | null = null;

function advance() {
  if (!playback.current) return stop();
  const tracks = library.tracksOf(playback.current.albumId);
  const next = playback.current.trackIndex + 1;
  if (next < tracks.length) {
    void playTrack(playback.current.albumId, next);
  } else {
    stop();
  }
}

function startTimer() {
  if (timer) clearInterval(timer);
  timer = setInterval(() => {
    if (!playback.isPlaying) return;
    playback.positionSec += TICK_MS / 1000;
    if (playback.positionSec >= playback.durationSec) {
      playback.positionSec = playback.durationSec;
      advance();
    }
  }, TICK_MS);
}
