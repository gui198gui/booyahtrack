// src/lib/destiny.ts

import { invoke } from "@tauri-apps/api/core";
import type {
  CurrentActivityResponse,
  PlayerSummary,
  SearchPlayerResponse
} from "./types";

export async function greetGuardian(name: string): Promise<string> {
  return await invoke<string>("greet", { name });
}

export async function getBungieStatus(): Promise<string> {
  return await invoke<string>("bungie_status");
}

export async function searchPlayerByBungieName(
  bungieName: string
): Promise<{
  raw: string;
  player: PlayerSummary | null;
}> {
  const raw = await invoke<string>("search_player", { bungieName });
  const parsed = JSON.parse(raw) as SearchPlayerResponse;

  if (!parsed?.Response || parsed.Response.length === 0) {
    return {
      raw,
      player: null
    };
  }

  const found = parsed.Response[0];

  return {
    raw,
    player: {
      displayName: found.displayName ?? "",
      bungieName: `${found.bungieGlobalDisplayName ?? ""}#${found.bungieGlobalDisplayNameCode ?? ""}`,
      membershipId: found.membershipId ?? "",
      membershipType: found.membershipType ?? 3
    }
  };
}

export async function getProfile(
  membershipId: string,
  membershipType: number
): Promise<string> {
  return await invoke<string>("get_profile", {
    membershipId,
    membershipType
  });
}

export async function getCurrentActivity(
  membershipId: string,
  membershipType: number
): Promise<{
  raw: string;
  data: CurrentActivityResponse;
}> {
  const raw = await invoke<string>("get_current_activity", {
    membershipId,
    membershipType
  });

  const data = JSON.parse(raw) as CurrentActivityResponse;

  return {
    raw,
    data
  };
}

export function resolveActivityLabel(modeType: number | null | undefined): string {
  if (modeType === null || modeType === undefined) {
    return "Offline / Sem atividade";
  }

  const map: Record<number, string> = {
    2: "Story",
    3: "Strike",
    4: "Raid",
    5: "All PvP",
    6: "Patrol",
    7: "All PvE",
    10: "Control",
    12: "Clash",
    15: "Nightfall",
    16: "Heroic Nightfall",
    18: "Iron Banner",
    40: "Social",
    48: "Rumble",
    63: "Dungeon",
    65: "Trials of Osiris",
    66: "Dares of Eternity",
    67: "Offensive",
    68: "Lost Sector",
    69: "Rift",
    70: "Zone Control",
    71: "Iron Banner Rift",
    72: "Relic"
  };

  return map[modeType] ?? `Mode Type ${modeType}`;
}

export function formatSeconds(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  const hh = String(hours).padStart(2, "0");
  const mm = String(minutes).padStart(2, "0");
  const ss = String(seconds).padStart(2, "0");

  return `${hh}:${mm}:${ss}`;
}