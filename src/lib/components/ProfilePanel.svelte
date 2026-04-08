<script lang="ts">
  export let foundBungieName = "";
  export let bungieName = "";
  export let onClose: () => void;
  export let onConfirm: () => void;
</script>

<section class="section-block profile-panel">
  <div class="panel-header">
    <h2>Welcome back</h2>
    <p>Want to switch accounts?</p>
  </div>

  <div class="saved-account">
    <div class="saved-account-main">
      <span class="platform-badge">🎮</span>
      <span>{foundBungieName || "No account selected"}</span>
    </div>
    <button class="mini-close" on:click={onClose}>×</button>
  </div>

  <input
    bind:value={bungieName}
    class="profile-input"
    placeholder="Add account: Name#1234"
  />

  <div class="profile-actions">
    <button class="confirm-button" on:click={onConfirm}>Confirm</button>
  </div>
</section>