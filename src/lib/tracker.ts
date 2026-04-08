// src/lib/tracker.ts

import { invoke } from "@tauri-apps/api/core";
import type { TodayRaidsResponse, TrackerSnapshot } from "./types";

export async function getTrackerSnapshot(
  membershipId: string,
  membershipType: number
): Promise<TrackerSnapshot> {
  await invoke("set_active_profile", {
    membershipId,
    membershipType
  });

  const raw = await invoke<string>("get_tracker_snapshot");
  return JSON.parse(raw) as TrackerSnapshot;
}

export async function getTodaysRaids(
  membershipId: string,
  membershipType: number
): Promise<TodayRaidsResponse> {
  await invoke("set_active_profile", {
    membershipId,
    membershipType
  });

  const raw = await invoke<string>("get_todays_raid_history");
  return JSON.parse(raw) as TodayRaidsResponse;
}