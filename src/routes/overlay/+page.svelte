<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getLiveElapsedSeconds } from "$lib/time";
  import type { TodayRaidsResponse, TrackerSnapshot } from "$lib/types";

  let timer = "00:00:00";
  let activity = "";
  let success = 0;
  let fail = 0;

  let snapshot: TrackerSnapshot | null = null;
  let raids: TodayRaidsResponse | null = null;
  let shouldShowOverlay = false;

  $: shouldShowOverlay =
    !!snapshot?.isInActivity &&
    !!snapshot?.activityStartedAt &&
    !!snapshot?.activityName;

  let fastPollHandle: ReturnType<typeof setInterval> | null = null;
  let slowPollHandle: ReturnType<typeof setInterval> | null = null;
  let timerHandle: ReturnType<typeof setInterval> | null = null;

  function formatSeconds(sec: number) {
    const safe = Math.max(0, Math.floor(sec));
    const h = Math.floor(safe / 3600);
    const m = Math.floor((safe % 3600) / 60);
    const s = safe % 60;

    return [
      h.toString().padStart(2, "0"),
      m.toString().padStart(2, "0"),
      s.toString().padStart(2, "0"),
    ].join(":");
  }

  function updateTimerDisplay() {
    if (snapshot?.isInActivity) {
      timer = formatSeconds(getLiveElapsedSeconds(snapshot));
    } else {
      timer = "00:00:00";
    }
  }

  function updateActivityLabel() {
    if (!snapshot?.isInActivity) {
      activity = "";
      return;
    }

    activity = snapshot.activityName?.trim() || "ACTIVITY";
  }

  function updateRaidStats() {
    if (!raids) {
      success = 0;
      fail = 0;
      return;
    }

    success = raids.activities.filter((a) => a.completed).length;
    fail = raids.activities.filter((a) => !a.completed).length;
  }

  async function ensureActiveProfile() {
    const raw = localStorage.getItem("activeProfile");
    if (!raw) return false;

    try {
      const profile = JSON.parse(raw) as {
        membershipId: string;
        membershipType: number;
      };

      if (!profile.membershipId?.trim()) return false;

      await invoke("set_active_profile", {
        membershipId: profile.membershipId,
        membershipType: profile.membershipType,
      });

      return true;
    } catch (err) {
      console.error("[overlay] failed to restore active profile:", err);
      return false;
    }
  }

  async function refreshSnapshotOnly() {
    const hasProfile = await ensureActiveProfile();

    if (!hasProfile) {
      snapshot = null;
      timer = "00:00:00";
      activity = "";
      return;
    }

    try {
      const snapshotRaw = await invoke<string>("get_tracker_snapshot");
      snapshot = JSON.parse(snapshotRaw) as TrackerSnapshot;
      updateActivityLabel();
      updateTimerDisplay();
    } catch (err) {
      console.error("[overlay] snapshot error:", err);
      snapshot = null;
      timer = "00:00:00";
      activity = "";
    }
  }

  async function refreshRaidsOnly() {
    const hasProfile = await ensureActiveProfile();

    if (!hasProfile) {
      raids = null;
      success = 0;
      fail = 0;
      return;
    }

    try {
      const raidsRaw = await invoke<string>("get_todays_raid_history");
      raids = JSON.parse(raidsRaw) as TodayRaidsResponse;
      updateRaidStats();
    } catch (err) {
      console.error("[overlay] raids error:", err);
      raids = null;
      success = 0;
      fail = 0;
    }
  }

  onMount(() => {
    void refreshSnapshotOnly();
    void refreshRaidsOnly();

    fastPollHandle = setInterval(() => {
      void refreshSnapshotOnly();
    }, 1000);

    slowPollHandle = setInterval(() => {
      void refreshRaidsOnly();
    }, 20000);

    timerHandle = setInterval(() => {
      updateTimerDisplay();
    }, 100);
  });

  onDestroy(() => {
    if (fastPollHandle) clearInterval(fastPollHandle);
    if (slowPollHandle) clearInterval(slowPollHandle);
    if (timerHandle) clearInterval(timerHandle);
  });
</script>

<main class="overlay-root">
  {#if shouldShowOverlay}
    <div class="timer">{timer}</div>
    <div class="activity-name">{activity}</div>
  {/if}

  <div class="stats">
    <span class="ok">● {success}</span>
    <span class="fail">● {fail}</span>
  </div>
</main>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
    border: 0;
    outline: none;
  }

  :global(*) {
    box-sizing: border-box;
  }

  .overlay-root {
    position: fixed;
    top: 0;
    left: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    pointer-events: none;
    user-select: none;
    background: transparent;
    border: 0;
    outline: none;
  }

  .timer {
    font-family: Inter, system-ui, sans-serif;
    font-size: 28px;
    font-weight: 800;
    line-height: 1;
    color: white;
    text-shadow:
      0 0 8px rgba(0, 0, 0, 0.95),
      0 0 16px rgba(0, 0, 0, 0.7);
  }

  .activity-name {
    font-family: Inter, system-ui, sans-serif;
    font-size: 11px;
    font-weight: 600;
    line-height: 1;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.82);
    text-shadow:
      0 0 6px rgba(0, 0, 0, 0.95),
      0 0 12px rgba(0, 0, 0, 0.65);
  }

  .stats {
    display: flex;
    gap: 10px;
    font-family: Inter, system-ui, sans-serif;
    font-size: 12px;
    font-weight: 700;
    line-height: 1;
    text-shadow:
      0 0 6px rgba(0, 0, 0, 0.95),
      0 0 12px rgba(0, 0, 0, 0.65);
  }

  .ok {
    color: #5cff87;
  }

  .fail {
    color: #ff5a5a;
  }
</style>
