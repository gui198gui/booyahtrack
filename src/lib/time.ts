// src/lib/time.ts

import type { TrackerSnapshot } from "./types";

export function getLiveElapsedSeconds(snapshot: TrackerSnapshot): number {
  if (!snapshot.isInActivity) return 0;

  const refreshedAtMs = new Date(snapshot.refreshedAt).getTime();
  const nowMs = Date.now();

  if (Number.isNaN(refreshedAtMs)) {
    return Math.max(0, snapshot.elapsedSeconds ?? 0);
  }

  const deltaSeconds = Math.max(0, Math.floor((nowMs - refreshedAtMs) / 1000));
  return Math.max(0, (snapshot.elapsedSeconds ?? 0) + deltaSeconds);
}