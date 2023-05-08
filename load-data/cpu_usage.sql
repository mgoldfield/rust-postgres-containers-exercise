-- dropping and recreating the database makes a clean install every time.  
-- if this was meant to update a persistent system, we would want to 
-- track migrations with a robust system and not rebuild everything each run.
DROP DATABASE IF EXISTS homework; 
CREATE DATABASE homework;
\c homework

CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS cpu_usage(
  ts    TIMESTAMPTZ,
  host  TEXT,
  usage DOUBLE PRECISION
);
SELECT create_hypertable('cpu_usage', 'ts', if_not_exists => true);

CREATE TABLE IF NOT EXISTS cpu_stats_queries(
  hostname TEXT,
  start_time TIMESTAMPTZ,
  end_time TIMESTAMPTZ
);
