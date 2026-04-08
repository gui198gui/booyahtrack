<script lang="ts">
    import { onMount } from "svelte";
    import { searchPlayerByBungieName } from "$lib/destiny";
    import { invoke } from "@tauri-apps/api/core";
    import { emit } from "@tauri-apps/api/event";

    type SavedProfile = {
        displayName?: string;
        bungieName?: string;
        membershipId: string;
        membershipType?: number;
    };

    let profiles: SavedProfile[] = [];
    let activeProfile: SavedProfile | null = null;

    let bungieName = "";
    let addStatus = "";

    function loadProfiles() {
        const savedRaw = localStorage.getItem("savedProfiles");
        const parsedProfiles = savedRaw
            ? (JSON.parse(savedRaw) as SavedProfile[])
            : [];

        profiles = parsedProfiles.map((p) => ({
            ...p,
            membershipId: String(p.membershipId),
            membershipType: p.membershipType ?? 3,
        }));

        const activeRaw = localStorage.getItem("activeProfile");

        if (activeRaw) {
            const parsedActive = JSON.parse(activeRaw) as SavedProfile;
            activeProfile = {
                ...parsedActive,
                membershipId: String(parsedActive.membershipId),
                membershipType: parsedActive.membershipType ?? 3,
            };
        } else {
            activeProfile = null;
        }
    }

    function saveProfiles() {
        localStorage.setItem("savedProfiles", JSON.stringify(profiles));
    }

    async function useProfile(profile: SavedProfile) {
        const normalizedProfile: SavedProfile = {
            displayName: profile.displayName,
            bungieName: profile.bungieName,
            membershipId: String(profile.membershipId),
            membershipType: profile.membershipType ?? 3,
        };

        activeProfile = { ...normalizedProfile };
        localStorage.setItem(
            "activeProfile",
            JSON.stringify(normalizedProfile),
        );

        await invoke("set_active_profile", {
            membershipId: normalizedProfile.membershipId,
            membershipType: normalizedProfile.membershipType,
        });

        await emit("profile-changed", normalizedProfile);

        // força refresh visual da lista/estado
        profiles = [...profiles];
    }

    function removeProfile(membershipId: string) {
        profiles = profiles.filter((p) => p.membershipId !== membershipId);
        saveProfiles();

        if (activeProfile?.membershipId === membershipId) {
            activeProfile = null;
            localStorage.removeItem("activeProfile");
        }
    }

    function isActive(profile: SavedProfile) {
        return (
            String(activeProfile?.membershipId ?? "") ===
            String(profile.membershipId)
        );
    }

    async function addProfile() {
        try {
            if (!bungieName.trim()) {
                addStatus = "Escreve um Bungie Name primeiro.";
                return;
            }

            addStatus = "A procurar jogador...";

            const result = await searchPlayerByBungieName(bungieName);

            if (!result.player) {
                addStatus = "Jogador não encontrado.";
                return;
            }

            const profile: SavedProfile = {
                displayName: result.player.displayName,
                bungieName: result.player.bungieName,
                membershipId: String(result.player.membershipId),
                membershipType: result.player.membershipType,
            };

            const alreadyExists = profiles.some(
                (p) => p.membershipId === profile.membershipId,
            );

            if (!alreadyExists) {
                profiles = [...profiles, profile];
                saveProfiles();
            }

            activeProfile = { ...profile };
            localStorage.setItem("activeProfile", JSON.stringify(profile));
            profiles = [...profiles];

            await invoke("set_active_profile", {
                membershipId: profile.membershipId,
                membershipType: profile.membershipType ?? 3,
            });
            await emit("profile-changed", profile);

            bungieName = "";
            addStatus = "Perfil adicionado com sucesso.";
        } catch (err) {
            addStatus = `Erro: ${err}`;
        }
    }

    onMount(() => {
        loadProfiles();
    });
</script>

<svelte:head>
    <title>Profiles</title>
</svelte:head>

<main class="profiles-page">
    <div class="page-header">
        <h1>Saved Profiles</h1>
        <p>Choose which account should be active in the main HUD.</p>
    </div>

    <section class="add-panel">
        <h2>Add account</h2>

        <input
            bind:value={bungieName}
            class="profile-input"
            placeholder="Name#1234"
        />

        <button class="add-button" on:click={addProfile}> Add </button>

        {#if addStatus}
            <p class="add-status">{addStatus}</p>
        {/if}
    </section>

    {#if profiles.length > 0}
        <div class="profiles-list">
            {#each profiles as profile}
                <div
                    class:profile-card-active={isActive(profile)}
                    class="profile-card"
                >
                    <div class="profile-main">
                        <div class="profile-top-row">
                            <div class="profile-name">
                                {profile.bungieName ||
                                    profile.displayName ||
                                    "Unknown account"}
                            </div>

                            {#if isActive(profile)}
                                <span class="active-badge">ACTIVE</span>
                            {/if}
                        </div>

                        <div class="profile-meta">
                            ID: {profile.membershipId}
                        </div>
                        <div class="profile-meta">
                            Type: {profile.membershipType ?? "-"}
                        </div>
                    </div>

                    <div class="profile-actions">
                        <button
                            class="use-button"
                            on:click={() => useProfile(profile)}
                            disabled={isActive(profile)}
                        >
                            {isActive(profile) ? "Using" : "Use"}
                        </button>

                        <button
                            class="remove-button"
                            on:click={() => removeProfile(profile.membershipId)}
                        >
                            Remove
                        </button>
                    </div>
                </div>
            {/each}
        </div>
    {:else}
        <div class="empty-state">
            <p>No saved profiles yet.</p>
        </div>
    {/if}
</main>

<style>
    :global(html, body) {
        margin: 0;
        padding: 0;
        font-family: Inter, system-ui, sans-serif;
        background: #0b1020;
        color: white;
    }

    .profiles-page {
        padding: 20px;
    }

    .page-header h1 {
        margin: 0 0 6px 0;
        font-size: 28px;
        font-weight: 800;
    }

    .page-header p {
        margin: 0 0 20px 0;
        color: rgba(255, 255, 255, 0.72);
        font-size: 14px;
    }

    .add-panel {
        margin-bottom: 18px;
        padding: 16px;
        border-radius: 16px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.06);
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .add-panel h2 {
        margin: 0;
        font-size: 18px;
    }

    .profile-input {
        border: 0;
        outline: none;
        border-radius: 12px;
        padding: 12px 14px;
        background: rgba(255, 255, 255, 0.08);
        color: white;
        font-size: 14px;
    }

    .add-button {
        border: 0;
        border-radius: 12px;
        padding: 12px 14px;
        font-size: 14px;
        font-weight: 700;
        cursor: pointer;
        background: #6d3cff;
        color: white;
    }

    .add-status {
        margin: 0;
        font-size: 13px;
        color: rgba(255, 255, 255, 0.78);
    }

    .profiles-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .profile-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 16px;
        padding: 16px;
        border-radius: 16px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .profile-card-active {
        border-color: rgba(92, 255, 135, 0.5);
        box-shadow: 0 0 0 1px rgba(92, 255, 135, 0.15) inset;
    }

    .profile-main {
        display: flex;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }

    .profile-top-row {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;
    }

    .profile-name {
        font-size: 17px;
        font-weight: 700;
    }

    .profile-meta {
        opacity: 0.72;
        font-size: 13px;
    }

    .active-badge {
        font-size: 11px;
        font-weight: 800;
        letter-spacing: 0.12em;
        color: #5cff87;
    }

    .profile-actions {
        display: flex;
        gap: 10px;
        flex-shrink: 0;
    }

    .use-button,
    .remove-button {
        border: 0;
        border-radius: 12px;
        padding: 10px 14px;
        font-size: 14px;
        font-weight: 700;
        cursor: pointer;
    }

    .use-button {
        background: #6d3cff;
        color: white;
    }

    .use-button:disabled {
        opacity: 0.6;
        cursor: default;
    }

    .remove-button {
        background: rgba(255, 255, 255, 0.08);
        color: white;
    }

    .empty-state {
        padding: 16px;
        border-radius: 16px;
        background: rgba(255, 255, 255, 0.04);
        color: rgba(255, 255, 255, 0.72);
    }
</style>
