-- ==============================================================================
-- Migration: Create report_records table for RoK Loot Watcher per-record cloud sync
-- Run this in your Supabase Dashboard -> SQL Editor
-- ==============================================================================

CREATE TABLE IF NOT EXISTS report_records (
    id          UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id     UUID REFERENCES auth.users(id) ON DELETE CASCADE NOT NULL,
    dedup_key   TEXT NOT NULL,
    mail_id     TEXT NOT NULL DEFAULT '',
    battle_type TEXT NOT NULL DEFAULT '',
    time_raw    BIGINT NOT NULL DEFAULT 0,
    time_utc    TEXT NOT NULL DEFAULT '',
    data        JSONB NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT now(),
    UNIQUE (user_id, dedup_key)
);

-- Row Level Security (RLS)
ALTER TABLE report_records ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can manage their own report records"
ON report_records
FOR ALL
USING (auth.uid() = user_id)
WITH CHECK (auth.uid() = user_id);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_report_records_user_time ON report_records (user_id, time_raw);
CREATE INDEX IF NOT EXISTS idx_report_records_user_type ON report_records (user_id, battle_type);

-- Grant Table Privileges to authenticated users
GRANT ALL ON TABLE report_records TO authenticated;
GRANT ALL ON TABLE report_records TO service_role;

