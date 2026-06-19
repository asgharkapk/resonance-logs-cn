import type { TeammateFantasyState } from "$lib/api";

type FantasyEntryIdentity = {
  summonerUuid: string;
  monsterId: number;
};

export function fantasyEntryKey(entry: FantasyEntryIdentity): string {
  return `${entry.summonerUuid}:${entry.monsterId}`;
}

export function withPreservedFantasySummonerName(
  entry: TeammateFantasyState,
  existing?: TeammateFantasyState,
): TeammateFantasyState {
  const summonerName = entry.summonerName ?? existing?.summonerName;
  return summonerName === undefined ? entry : { ...entry, summonerName };
}
