<script lang="ts">
  export let trackerStatus = "Parado";
  export let currentStateLabel = "Desconhecido";
  export let activeBungieName = "No account selected";
  export let currentCharacterId = "";
  export let isAutoTracking = false;
  export let onStartTracking: () => void;
  export let onStopTracking: () => void;
</script>

<section class="section-block live-status">
  <div class="status-grid">
    <div>
      <span class="status-label">Status</span>
      <p>{trackerStatus}</p>
    </div>

    <div>
      <span class="status-label">State</span>
      <p>{currentStateLabel}</p>
    </div>

    <div>
      <span class="status-label">Account</span>
      <p>{activeBungieName}</p>
    </div>

    <div>
      <span class="status-label">Character</span>
      <p>{currentCharacterId || "-"}</p>
    </div>
  </div>

  <div class="action-row">
    <button
      class="primary-button"
      on:click={onStartTracking}
      disabled={isAutoTracking}
    >
      Start tracking
    </button>

    <button
      class="secondary-button"
      on:click={onStopTracking}
      disabled={!isAutoTracking}
    >
      Stop tracking
    </button>
  </div>
</section>