CREATE TABLE IF NOT EXISTS pixel_hits (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ip_address TEXT NOT NULL,
  user_agent TEXT,
  camo_id TEXT,
  timestamp DATETIME NOT NULL
);
