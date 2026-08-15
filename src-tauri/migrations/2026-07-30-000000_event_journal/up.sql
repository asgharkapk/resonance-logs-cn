DROP TABLE IF EXISTS encounter_events;
DROP TABLE IF EXISTS encounter_timeline;
DROP TABLE IF EXISTS encounter_combat_log;
DROP TABLE IF EXISTS encounter_data;

ALTER TABLE encounters
  ADD COLUMN quality_flags INTEGER NOT NULL DEFAULT 0;
ALTER TABLE encounters
  ADD COLUMN total_dmg_exact TEXT;
ALTER TABLE encounters
  ADD COLUMN total_heal_exact TEXT;

CREATE TABLE encounter_event_chunks (
  encounter_id INTEGER NOT NULL,
  stream_kind TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  first_sequence INTEGER NOT NULL,
  last_sequence INTEGER NOT NULL,
  start_offset_ms INTEGER NOT NULL,
  end_offset_ms_exclusive INTEGER NOT NULL,
  event_count INTEGER NOT NULL,
  data BLOB NOT NULL,
  PRIMARY KEY(encounter_id, stream_kind, chunk_index),
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE,
  CHECK(length(stream_kind) > 0),
  CHECK(chunk_index >= 0),
  CHECK(first_sequence >= 0),
  CHECK(last_sequence >= first_sequence),
  CHECK(start_offset_ms >= 0),
  CHECK(end_offset_ms_exclusive > start_offset_ms),
  CHECK(event_count > 0),
  CHECK(event_count <= last_sequence - first_sequence + 1),
  CHECK(length(data) > 0)
);

-- Chunks and UI selections both use half-open intervals.
CREATE INDEX idx_encounter_event_chunks_range
  ON encounter_event_chunks(
    encounter_id,
    start_offset_ms,
    end_offset_ms_exclusive
  );

CREATE INDEX idx_encounter_event_chunks_sequence
  ON encounter_event_chunks(encounter_id, first_sequence, last_sequence);

CREATE TABLE encounter_projection (
  encounter_id INTEGER PRIMARY KEY NOT NULL,
  last_sequence INTEGER NOT NULL,
  quality_flags INTEGER NOT NULL,
  data BLOB NOT NULL,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE,
  CHECK(last_sequence >= 0),
  CHECK(quality_flags >= 0),
  CHECK(length(data) > 0)
);
