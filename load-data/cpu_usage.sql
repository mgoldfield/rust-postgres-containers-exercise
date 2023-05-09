-- dropping and recreating the database makes a clean install every time.  
-- if this was meant to update a persistent system, we would want to 
-- track migrations with a robust system and not rebuild everything each run.
DROP DATABASE IF EXISTS homework; 
CREATE DATABASE homework;
\c homework

CREATE TABLE IF NOT EXISTS cpu_usage(
  ts    TIMESTAMP NOT NULL,
  host  TEXT NOT NULL,
  usage DOUBLE PRECISION NOT NULL
);

CREATE INDEX truncatedts_idx on cpu_usage (date_trunc('minute', ts));
SELECT create_hypertable('cpu_usage', 'ts', if_not_exists => true);

CREATE TABLE IF NOT EXISTS cpu_stats_queries(
  hostname TEXT NOT NULL,
  start_time TIMESTAMP NOT NULL,
  end_time TIMESTAMP NOT NULL
);
