DROP TABLE IF EXISTS encounter_projection;
DROP TABLE IF EXISTS encounter_event_chunks;

ALTER TABLE encounters DROP COLUMN total_heal_exact;
ALTER TABLE encounters DROP COLUMN total_dmg_exact;
ALTER TABLE encounters DROP COLUMN quality_flags;
