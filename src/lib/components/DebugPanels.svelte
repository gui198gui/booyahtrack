<script lang="ts">
  import type { TrackerSnapshot } from "$lib/types";

  export let snapshotData: TrackerSnapshot | null = null;
  export let snapshotResult = "";
  export let todaysRaidsResult = "";
  export let message = "";
  export let playerResult = "";
  export let profileResult = "";
  export let apiResult = "";

  export let name = "";
  export let bungieName = "";
  export let membershipId = "";
  export let membershipType = 3;

  export let onTestRust: () => void;
  export let onSearchPlayer: () => void;
  export let onLoadProfile: () => void;
  export let onTestBungie: () => void;
</script>

<section class="debug-grid">
  <section class="section-block debug-card">
    <h2>Tracker Snapshot</h2>

    {#if snapshotData}
      <div class="mini-stats">
        <p><strong>In Activity:</strong> {snapshotData.isInActivity ? "Sim" : "Não"}</p>
        <p><strong>Character ID:</strong> {snapshotData.characterId}</p>
        <p><strong>Mode Type:</strong> {snapshotData.currentActivityModeType ?? "-"}</p>
        <p><strong>Refresh:</strong> {snapshotData.refreshedAt}</p>
      </div>
    {/if}

    {#if snapshotResult}
      <pre>{snapshotResult}</pre>
    {/if}
  </section>

  <section class="section-block debug-card">
    <h2>Raids de Hoje Raw</h2>
    {#if todaysRaidsResult}
      <pre>{todaysRaidsResult}</pre>
    {/if}
  </section>

  <section class="section-block debug-card">
    <h2>Manual Tools</h2>

    <div class="tool-column">
      <input bind:value={name} placeholder="O teu nome" />
      <button on:click={onTestRust}>Testar Rust</button>

      <input bind:value={bungieName} placeholder="Nome#1234" />
      <button on:click={onSearchPlayer}>Procurar jogador</button>

      <label for="membership-id">Membership ID</label>
      <input id="membership-id" bind:value={membershipId} placeholder="Membership ID" />

      <label for="membership-type">Membership Type</label>
      <input id="membership-type" bind:value={membershipType} type="number" />

      <button on:click={onLoadProfile}>Ver perfil</button>
      <button on:click={onTestBungie}>Testar Bungie API</button>
    </div>

    {#if message}
      <pre>{message}</pre>
    {/if}

    {#if playerResult}
      <pre>{playerResult}</pre>
    {/if}

    {#if profileResult}
      <pre>{profileResult}</pre>
    {/if}

    {#if apiResult}
      <pre>{apiResult}</pre>
    {/if}
  </section>
</section>