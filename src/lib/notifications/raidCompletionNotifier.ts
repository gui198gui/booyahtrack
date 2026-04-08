import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from "@tauri-apps/plugin-notification";

import type { TodayRaidActivity } from "$lib/types";

function formatDuration(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds || 0));
  const h = Math.floor(safe / 3600);
  const m = Math.floor((safe % 3600) / 60);
  const s = safe % 60;

  return [
    h.toString().padStart(2, "0"),
    m.toString().padStart(2, "0"),
    s.toString().padStart(2, "0")
  ].join(":");
}

async function ensureNotificationPermission(): Promise<boolean> {
  let granted = await isPermissionGranted();

  if (!granted) {
    const permission = await requestPermission();
    granted = permission === "granted";
  }

  return granted;
}

export function createRaidCompletionNotifier() {
  let initialized = false;
  const seenCompletedIds = new Set<string>();

  async function notifyRaid(raid: TodayRaidActivity) {
    const allowed = await ensureNotificationPermission();
    if (!allowed) return;

    await sendNotification({
      title: "Raid concluída",
      body: `${raid.activityName} • ${formatDuration(raid.durationSeconds)}`
    });
  }

  return {
    async update(activities: TodayRaidActivity[] | undefined | null) {
      if (!activities) return;

      const completed = activities.filter((a) => a.completed);

      // primeiro load: só semear estado, sem notificar
      if (!initialized) {
        for (const raid of completed) {
          seenCompletedIds.add(raid.instanceId);
        }
        initialized = true;
        return;
      }

      const newCompleted = completed.filter(
        (raid) => !seenCompletedIds.has(raid.instanceId)
      );

      // marcar como vistos antes de notificar
      for (const raid of completed) {
        seenCompletedIds.add(raid.instanceId);
      }

      // opcional: ordem antiga -> nova
      for (const raid of [...newCompleted].reverse()) {
        try {
          await notifyRaid(raid);
        } catch (err) {
          console.error("[raid-notify] failed:", err);
        }
      }
    },

    reset() {
      initialized = false;
      seenCompletedIds.clear();
    }
  };
}