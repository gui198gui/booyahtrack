<script lang="ts">
  import "../app.css";
  import HudTopbar from "$lib/components/HudTopbar.svelte";
  import ActivitySummary from "$lib/components/ActivitySummary.svelte";
  import LiveStatus from "$lib/components/LiveStatus.svelte";
  import ProfilePanel from "$lib/components/ProfilePanel.svelte";
  import DebugPanels from "$lib/components/DebugPanels.svelte";
  import {
    formatSeconds,
    getBungieStatus,
    getProfile,
    greetGuardian,
    resolveActivityLabel,
    searchPlayerByBungieName,
  } from "$lib/destiny";
  import { getTodaysRaids, getTrackerSnapshot } from "$lib/tracker";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { createRaidCompletionNotifier } from "$lib/notifications/raidCompletionNotifier";
  import { getLiveElapsedSeconds } from "$lib/time";
  import type { TodayRaidsResponse, TrackerSnapshot } from "$lib/types";
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

  import { invoke } from "@tauri-apps/api/core";
  let name = "Guardian";
  let message = "";

  let apiResult = "";
  let bungieName = "";
  let playerResult = "";

  let membershipId = "";
  let membershipType = 3;
  let profileResult = "";
  let unlistenProfileChanged: UnlistenFn | null = null;
  let foundDisplayName = "";
  let foundBungieName = "";
  let currentStateLabel = "Desconhecido";
  let isRefreshing = false;
  let snapshotRequestSeq = 0;
  let latestAppliedSnapshotSeq = 0;
  let isAutoTracking = false;
  let fastPollingHandle: ReturnType<typeof setInterval> | null = null;
  let slowPollingHandle: ReturnType<typeof setInterval> | null = null;

  let activityElapsedSeconds = 0;
  let timerHandle: ReturnType<typeof setInterval> | null = null;

  let lastActivityInstanceKey = "";
  let trackerStatus = "Parado";

  let snapshotResult = "";
  let snapshotData: TrackerSnapshot | null = null;
  let currentCharacterId = "";
  let currentActivityHash = "";
  let currentActivityModeHash = "";
  let currentActivityModeType = "";
  let todaysRaidsResult = "";
  let todaysRaidsData: TodayRaidsResponse | null = null;

  let showDebugPanels = false;
  let showProfileSwitcher = false;
  const raidNotifier = createRaidCompletionNotifier();
  let shouldShowTimer = false;
  $: shouldShowTimer =
    !!snapshotData?.isInActivity &&
    !!snapshotData?.activityStartedAt &&
    !!snapshotData?.activityName;

  $: successCount = todaysRaidsData
    ? todaysRaidsData.activities.filter((a) => a.completed).length
    : 0;

  $: failCount = todaysRaidsData
    ? todaysRaidsData.activities.filter((a) => !a.completed).length
    : 0;

  $: currentActivityTitle =
    shouldShowTimer &&
    currentStateLabel &&
    currentStateLabel !== "Offline / Sem atividade"
      ? currentStateLabel.toUpperCase()
      : "NO ACTIVITY";
  $: displayElapsedSeconds = shouldShowTimer ? activityElapsedSeconds : 0;

  $: activeBungieName = foundBungieName || "No account selected";

  async function testRust() {
    message = await greetGuardian(name);
  }
  async function toggleOverlay() {
    const overlay = await WebviewWindow.getByLabel("overlay");

    if (!overlay) {
      console.error("Overlay window não encontrada.");
      return;
    }

    const visible = await overlay.isVisible();

    if (visible) {
      await overlay.hide();
    } else {
      await overlay.show();
      await overlay.setAlwaysOnTop(true);
    }
  }
  async function applyActiveProfileFromEvent(profile: {
    bungieName?: string;
    membershipId: string;
    membershipType?: number;
  }) {
    membershipId = profile.membershipId;
    membershipType = profile.membershipType ?? 3;
    foundBungieName = profile.bungieName || "";
    foundDisplayName = profile.bungieName || "";

    trackerStatus = "A atualizar perfil...";

    await loadSnapshot();
    await loadTodaysRaids();

    if (!isAutoTracking) {
      startAutoTracking();
    }
  }
  async function openProfilesWindow() {
    const existing = await WebviewWindow.getByLabel("profiles");

    if (existing) {
      await existing.show();
      await existing.setFocus();
      return;
    }

    const win = new WebviewWindow("profiles", {
      url: "/profiles",
      title: "Profiles",
      width: 520,
      height: 680,
      resizable: false,
      center: true,
      focus: true,
    });

    win.once("tauri://created", () => {
      console.log("[profiles] window created");
    });

    win.once("tauri://error", (e) => {
      console.error("[profiles] failed to create window", e);
    });
  }
  async function testBungie() {
    try {
      apiResult = "A pedir dados...";
      const result = await getBungieStatus();
      apiResult = result.slice(0, 500) + "...";
    } catch (error) {
      apiResult = `Erro: ${error}`;
    }
  }
  function saveCurrentProfile() {
    if (!membershipId.trim()) return;

    const profile = {
      bungieName: foundBungieName || bungieName,
      membershipId,
      membershipType,
    };

    localStorage.setItem("activeProfile", JSON.stringify(profile));

    const raw = localStorage.getItem("savedProfiles");
    const savedProfiles = raw ? JSON.parse(raw) : [];

    const alreadyExists = savedProfiles.some(
      (p: { membershipId: string }) => p.membershipId === profile.membershipId,
    );

    if (!alreadyExists) {
      savedProfiles.push(profile);
      localStorage.setItem("savedProfiles", JSON.stringify(savedProfiles));
    }
  }

  async function searchPlayer() {
    try {
      playerResult = "A procurar jogador...";
      foundDisplayName = "";
      foundBungieName = "";

      const result = await searchPlayerByBungieName(bungieName);

      playerResult = result.raw;

      if (result.player) {
        membershipId = result.player.membershipId;
        membershipType = result.player.membershipType;
        foundDisplayName = result.player.displayName;
        foundBungieName = result.player.bungieName;

        await invoke("set_active_profile", {
          membershipId,
          membershipType,
        });

        saveCurrentProfile();

        await loadSnapshot();
        await loadTodaysRaids();
        startAutoTracking();
      }
    } catch (err) {
      playerResult = `Erro: ${err}`;
    }
  }

  async function refreshAll() {
    await loadSnapshot();
    await loadTodaysRaids();
  }

  async function startAutoTracking() {
    if (!membershipId.trim()) {
      trackerStatus = "Preenche primeiro o jogador";
      return;
    }

    if (isAutoTracking) return;

    isAutoTracking = true;
    trackerStatus = "Tracking automático ativo";

    await loadSnapshot();
    await loadTodaysRaids();

    fastPollingHandle = setInterval(() => {
      void loadSnapshot();
    }, 1000);

    slowPollingHandle = setInterval(() => {
      void loadTodaysRaids();
    }, 20000);
  }

  function stopAutoTracking() {
    isAutoTracking = false;
    trackerStatus = "Tracking parado";

    if (fastPollingHandle) {
      clearInterval(fastPollingHandle);
      fastPollingHandle = null;
    }

    if (slowPollingHandle) {
      clearInterval(slowPollingHandle);
      slowPollingHandle = null;
    }
  }

  async function loadSnapshot() {
    const requestSeq = ++snapshotRequestSeq;

    try {
      snapshotResult = "A buscar snapshot...";
      const nextSnapshot = await getTrackerSnapshot(
        membershipId,
        membershipType,
      );

      if (requestSeq < latestAppliedSnapshotSeq) {
        return;
      }

      latestAppliedSnapshotSeq = requestSeq;
      snapshotData = nextSnapshot;
      snapshotResult = JSON.stringify(snapshotData, null, 2);

      currentCharacterId = snapshotData.characterId ?? "";
      currentActivityHash =
        snapshotData.currentActivityHash?.toString?.() ?? "";
      currentActivityModeHash =
        snapshotData.currentActivityModeHash?.toString?.() ?? "";
      currentActivityModeType =
        snapshotData.currentActivityModeType?.toString?.() ?? "";

      currentStateLabel = resolveActivityLabel(
        snapshotData.currentActivityModeType,
      );

      if (snapshotData?.isInActivity) {
        const newInstanceKey = snapshotData.activityInstanceKey ?? "";

        if (!newInstanceKey) {
          lastActivityInstanceKey = "";
        } else if (newInstanceKey !== lastActivityInstanceKey) {
          lastActivityInstanceKey = newInstanceKey;
          trackerStatus = "Nova atividade detetada";

          setTimeout(() => {
            if (trackerStatus === "Nova atividade detetada") {
              trackerStatus = "A acompanhar atividade";
            }
          }, 2000);
        }

        const nextElapsed = getLiveElapsedSeconds(snapshotData);

        if (
          Math.abs(nextElapsed - activityElapsedSeconds) > 10 &&
          lastActivityInstanceKey === snapshotData.activityInstanceKey
        ) {
          // ignora salto estranho
        } else {
          activityElapsedSeconds = nextElapsed;
        }
        startTimer();
      } else {
        lastActivityInstanceKey = "";
        trackerStatus = "Sem atividade ativa";
        resetActivityTimer();
      }
    } catch (err) {
      snapshotResult = `Erro: ${err}`;
      trackerStatus = "Erro no snapshot";
    }
  }

  async function loadProfile() {
    try {
      if (!membershipId.trim()) {
        profileResult = "Erro: falta o Membership ID.";
        return;
      }

      profileResult = "A buscar perfil...";
      const result = await getProfile(membershipId, membershipType);
      profileResult = result;
    } catch (err) {
      profileResult = `Erro: ${err}`;
    }
  }

  function stopTimer() {
    if (timerHandle) {
      clearInterval(timerHandle);
      timerHandle = null;
    }
  }

  function startTimer() {
    stopTimer();

    if (!snapshotData?.isInActivity) {
      activityElapsedSeconds = 0;
      return;
    }

    activityElapsedSeconds = getLiveElapsedSeconds(snapshotData);

    timerHandle = setInterval(() => {
      if (!snapshotData?.isInActivity) {
        activityElapsedSeconds = 0;
        return;
      }

      activityElapsedSeconds = getLiveElapsedSeconds(snapshotData);
    }, 50);
  }

  function resetActivityTimer() {
    activityElapsedSeconds = 0;
    stopTimer();
  }

  function formatDuration(durationSeconds: number) {
    const minutes = Math.floor(durationSeconds / 60);
    const seconds = durationSeconds % 60;
    return `${minutes}m ${seconds}s`;
  }

  function formatTimeAgo(period: string) {
    const diffMs = Date.now() - new Date(period).getTime();
    const diffMinutes = Math.floor(diffMs / 60000);

    if (diffMinutes < 1) return "just now";
    if (diffMinutes < 60) return `${diffMinutes}m ago`;

    const diffHours = Math.floor(diffMinutes / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  }

  async function loadTodaysRaids() {
    try {
      if (!membershipId.trim()) {
        todaysRaidsResult = "Erro: falta o Membership ID.";
        return;
      }

      todaysRaidsResult = "A buscar raids de hoje...";
      todaysRaidsData = await getTodaysRaids(membershipId, membershipType);
      todaysRaidsResult = JSON.stringify(todaysRaidsData, null, 2);
    } catch (err) {
      todaysRaidsResult = `Erro: ${err}`;
    }
  }

  function toggleDebugPanels() {
    showDebugPanels = !showDebugPanels;
  }

  function toggleProfileSwitcher() {
    showProfileSwitcher = !showProfileSwitcher;
  }
  onMount(() => {
    void (async () => {
      unlistenProfileChanged = await listen(
        "profile-changed",
        async (event) => {
          const profile = event.payload as {
            bungieName?: string;
            membershipId: string;
            membershipType?: number;
          };

          await applyActiveProfileFromEvent(profile);
        },
      );
    })();
  });

  onDestroy(() => {
    if (unlistenProfileChanged) {
      unlistenProfileChanged();
      unlistenProfileChanged = null;
    }
    stopAutoTracking();
    stopTimer();
  });
</script>

<!-- src/routes/+page.svelte -->
<svelte:head>
  <title>Destiny Tracker</title>
</svelte:head>

<main class="page-shell">
  <section class="hud-card">
    <HudTopbar
      displayTime={formatSeconds(displayElapsedSeconds)}
      activityTitle={currentActivityTitle}
      onToggleProfiles={openProfilesWindow}
    />

    <ActivitySummary
      {todaysRaidsData}
      {successCount}
      {failCount}
      {formatDuration}
      {formatTimeAgo}
    />

    <LiveStatus
      {trackerStatus}
      {currentStateLabel}
      {activeBungieName}
      {currentCharacterId}
      {isAutoTracking}
      onStartTracking={startAutoTracking}
      onStopTracking={stopAutoTracking}
    />

    {#if showProfileSwitcher}
      <ProfilePanel
        {foundBungieName}
        bind:bungieName
        onClose={toggleProfileSwitcher}
        onConfirm={searchPlayer}
      />
    {/if}

    {#if showDebugPanels}
      <DebugPanels
        {snapshotData}
        {snapshotResult}
        {todaysRaidsResult}
        {message}
        {playerResult}
        {profileResult}
        {apiResult}
        bind:name
        bind:bungieName
        bind:membershipId
        bind:membershipType
        onTestRust={testRust}
        onSearchPlayer={searchPlayer}
        onLoadProfile={loadProfile}
        onTestBungie={testBungie}
      />
    {/if}
  </section>
</main>
