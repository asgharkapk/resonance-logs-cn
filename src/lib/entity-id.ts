export type EntityId = string;

const ENTITY_UID_SHIFT = 16n;

export function uidFromEntityUuid(entityUuid: EntityId): number {
  const uuid = BigInt(entityUuid);
  return Number(uuid >> ENTITY_UID_SHIFT);
}

export function displayEntityId(entityUuid: EntityId): string {
  return `#${uidFromEntityUuid(entityUuid)}`;
}
