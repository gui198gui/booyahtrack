<script lang="ts">
  export let displayTime = "00:00:00";
  export let activityTitle = "NO ACTIVITY";
  export let onToggleProfiles: () => void;
</script>

<header class="hud-topbar">
  <div class="hud-title-block">
    <div class="timer-display">{displayTime}</div>
    <div class="activity-title">{activityTitle}</div>
  </div>

  <div class="top-actions">
    <button class="icon-button" on:click={onToggleProfiles} title="Profiles">
      👤
    </button>
  </div>
</header>