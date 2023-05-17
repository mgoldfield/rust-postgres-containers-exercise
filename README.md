to build: `docker compose build`

to run: `docker compose run benchmark-queries`

to test: `docker compose run benchmark-queries-test`

Both `benchmark-queries` and `benchmark-queries-test` can be run with the `--build` flag as well.

## Design Considerations

I know there are a lot of avenues to approach designing, optimizing, and making this program more robust. I took a look at a few of these here, and tried to document what I could around other potential work.

### Data

Since there aren't timestamps on the data, I made the decision to move to the `TIMESTAMP` type from the `TIMESTAMPTZ` type.
With production data, we'd need to make sure our assumption of a timestamp without a timezone was correct.

I decided to load the `query_params` data into the database because I think that allows for the easiest usage and manipulation of the data. It also paves the way for extending functionality for different scenarios, like queuing new `query_params` requests.

### Robustness

Currently, data is loaded in the `load-data` container. If the data is not formatted correctly, this container will exit with a failure code and run will stop. Because I am trying to keep things simple, I am not adding much around fault tolerance and recoverability. I'd like to note that in a production setting we could produce and archive wal logs, and have things like hot standby replicas for fault tolerance.

### Indexes

Adding indexes in varying complexity to the `cpu_usage` table allow us to lookup data faster.
Making `cpu_usage` a hypertable as we configured it automatically makes an index on `ts`.

Using this example query:

```sql
explain analyze
select max(usage) as max, min(usage) as min, date_trunc('minute', ts) as minute
from cpu_usage
where host = 'host_000009'
    and ts >= '2017-01-02 07:42:00'::timestamp
    and ts < '2017-01-02 08:42:00'::timestamp
group by date_trunc('minute', ts);
```

With no additional index:

```
                                                                                   QUERY PLAN
--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 GroupAggregate  (cost=0.42..234.98 rows=200 width=24) (actual time=0.071..1.481 rows=61 loops=1)
   Group Key: (date_trunc('minute'::text, _hyper_1_1_chunk.ts))
   ->  Result  (cost=0.42..229.81 rows=356 width=16) (actual time=0.040..1.419 rows=360 loops=1)
         ->  Index Scan Backward using _hyper_1_1_chunk_cpu_usage_ts_idx on _hyper_1_1_chunk  (cost=0.42..225.36 rows=356 width=16) (actual time=0.039..1.388 rows=360 loops=1)
               Index Cond: ((ts >= '2017-01-01 08:52:14'::timestamp without time zone) AND (ts < '2017-01-01 09:52:14'::timestamp without time zone))
               Filter: (host = 'host_000003'::text)
               Rows Removed by Filter: 6840
```

My two main thoughts to optimize were:

- speed up the GroupAggregate by precomputing the truncated date
- moving the filter on host to be using an index of some kind

Adding an index on `(ts, host)` yeilds a more streamlined query plan for the longer running queries

```
                                                                                      QUERY PLAN
-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
 GroupAggregate  (cost=0.42..35.32 rows=7 width=24)
   Group Key: (date_trunc('minute'::text, _hyper_1_1_chunk.ts))
   ->  Result  (cost=0.42..35.18 rows=7 width=16)
         ->  Index Scan using _hyper_1_1_chunk_truncatedts_idx on _hyper_1_1_chunk  (cost=0.42..35.09 rows=7 width=16)
               Index Cond: ((ts >= '2017-01-02 07:42:00'::timestamp without time zone) AND (ts < '2017-01-02 08:42:00'::timestamp without time zone) AND (host = 'host_000009'::text))
```

Adding an index using `date_trunc('minute', ts)` does not result in a change to the query plans or benchmarks for queries I investigated. I think heavier load scenarios could benefit from more index investigation.
