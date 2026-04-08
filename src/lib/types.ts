// src/lib/types.ts

export type SearchPlayerResponse = {
  Response?: Array<{
    membershipId?: string;
    membershipType?: number;
    displayName?: string;
    bungieGlobalDisplayName?: string;
    bungieGlobalDisplayNameCode?: number;
  }>;
  ErrorCode?: number;
  ErrorStatus?: string;
  Message?: string;
};

export type CurrentActivityResponse = {
  characterId?: string;
  profileLastPlayed?: string;
  characterLastPlayed?: string;
  currentActivityHash?: number | null;
  currentActivityModeHash?: number | null;
  currentActivityModeType?: number | null;
  activityStartedAt?: string | null;
  elapsedSeconds?: number;
  timerSource?: "dateActivityStarted" | null;
  refreshedAt?: string;
};

export type PlayerSummary = {
  displayName: string;
  bungieName: string;
  membershipId: string;
  membershipType: number;
};

export type TrackerSnapshot = {
  membershipId: string;
  membershipType: number;
  characterId: string;
  profileLastPlayed: string;
  characterLastPlayed: string;
  currentActivityHash: number | null;
  currentActivityModeHash: number | null;
  currentActivityModeType: number | null;
  isInActivity: boolean;
  activityStartedAt: string | null;
  activityInstanceKey: string;
  activityName: string | null;
  elapsedSeconds: number;
  timerSource: "dateActivityStarted" | null;
  refreshedAt: string;
};

export type TodayRaidActivity = {
  instanceId: string;
  characterId: string;
  period: string;
  activityHash: number;
  activityName: string;
  durationSeconds: number;
  completed: boolean;
};

export type TodayRaidsResponse = {
  resetAt: string;
  now: string;
  totalRaidsToday: number;
  activities: TodayRaidActivity[];
};