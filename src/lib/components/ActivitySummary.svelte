<script lang="ts">
  import { formatSeconds } from "$lib/destiny";
  import type { TodayRaidsResponse } from "$lib/types";

  export let todaysRaidsData: TodayRaidsResponse | null = null;
  export let successCount = 0;
  export let failCount = 0;
  export let formatDuration: (seconds: number) => string;
  export let formatTimeAgo: (value: string) => string;
</script>

<section class="section-block summary-block">
  <div class="summary-head">
    <h2>Today's activities</h2>

    <div class="summary-dots">
      <span class="dot-group success">
        <span class="dot"></span>
        {successCount}
      </span>

      <span class="dot-group fail">
        <span class="dot"></span>
        {failCount}
      </span>
    </div>
  </div>

  {#if todaysRaidsData && todaysRaidsData.activities.length > 0}
    <div class="activity-list">
      {#each todaysRaidsData.activities as activity}
        <div class="activity-item">
          <div
            class="left-marker {activity.completed ? 'completed' : 'failed'}"
          ></div>

          <div class="activity-item-content">
            <div class="activity-row-top">
              <p class="activity-name">{activity.activityName}</p>
              <button class="open-button" title="Future details action">↗</button>
            </div>

            <p class="activity-subtext">
              {formatDuration(activity.durationSeconds)} • {formatTimeAgo(activity.period)}
            </p>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <p>No activities found for today yet.</p>
    </div>
  {/if}
</section>