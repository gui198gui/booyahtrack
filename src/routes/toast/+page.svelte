<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { TodayRaidsResponse } from "$lib/types";

  let visible = false;
  let title = "";
  let duration = "";

  let pollHandle: ReturnType<typeof setInterval> | null = null;
  let hideHandle: ReturnType<typeof setTimeout> | null = null;

  const seenCompletedRaidIds = new Set<string>();
  let initialized = false;

  function formatDuration(totalSeconds: number) {
    const safe = Math.max(0, Math.floor(totalSeconds || 0));
    const h = Math.floor(safe / 3600);
    const m = Math.floor((safe % 3600) / 60);
    const s = safe % 60;

    if (h > 0) return `${h}h ${m}m ${s}s`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }

  function showToast(activityName: string, durationSeconds: number) {
    title = activityName;
    duration = formatDuration(durationSeconds);
    visible = true;

    if (hideHandle) clearTimeout(hideHandle);

    hideHandle = setTimeout(() => {
      visible = false;
    }, 5000);
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
        membershipType: profile.membershipType
      });

      return true;
    } catch (err) {
      console.error("[toast] failed to restore active profile:", err);
      return false;
    }
  }

  async function refreshRaidsForToast() {
    const hasProfile = await ensureActiveProfile();
    if (!hasProfile) return;

    try {
      const raidsRaw = await invoke<string>("get_todays_raid_history");
      const raids = JSON.parse(raidsRaw) as TodayRaidsResponse;

      const completed = raids.activities.filter((a) => a.completed);

      if (!initialized) {
        for (const raid of completed) {
          seenCompletedRaidIds.add(raid.instanceId);
        }
        initialized = true;
        return;
      }

      const newCompleted = completed.filter(
        (raid) => !seenCompletedRaidIds.has(raid.instanceId)
      );

      for (const raid of completed) {
        seenCompletedRaidIds.add(raid.instanceId);
      }

      for (const raid of [...newCompleted].reverse()) {
        showToast(raid.activityName, raid.durationSeconds);
      }
    } catch (err) {
      console.error("[toast] raids error:", err);
    }
  }

  onMount(() => {
    void refreshRaidsForToast();

    pollHandle = setInterval(() => {
      void refreshRaidsForToast();
    }, 20000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
    if (hideHandle) clearTimeout(hideHandle);
  });
</script>

{#if visible}
  <div class="toast">
    <div class="toast-accent"></div>

    <div class="toast-content">
      <div class="toast-label">RAID COMPLETE</div>
      <div class="toast-title">{title}</div>
      <div class="toast-meta">Duration: {duration}</div>
    </div>
  </div>
{/if}

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
  }

  .toast {
    position: fixed;
    right: 0;
    bottom: 0;
    width: 340px;
    min-height: 84px;
    display: flex;
    overflow: hidden;
    border-radius: 16px;
    background: rgba(10, 10, 14, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow:
      0 10px 30px rgba(0, 0, 0, 0.4),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
    color: white;
    font-family: Inter, system-ui, sans-serif;
    pointer-events: none;
    user-select: none;
    box-sizing: border-box;
    opacity: 0;
    transform: translateY(20px) scale(0.98);
    animation: toast-in 220ms ease forwards;
  }

  .toast-accent {
    width: 4px;
    background: linear-gradient(180deg, #5cff87, #3b82f6);
    flex-shrink: 0;
  }

  .toast-content {
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    box-sizing: border-box;
    width: 100%;
  }

  .toast-label {
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.16em;
    color: rgba(255, 255, 255, 0.62);
  }

  .toast-title {
    font-size: 20px;
    font-weight: 800;
    line-height: 1.1;
    color: white;
  }

  .toast-meta {
    margin-top: 4px;
    font-size: 13px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.76);
  }

  @keyframes toast-in {
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>