to build: `docker compose build`

to run: `docker compose run benchmark-queries`

to test: `docker compose run --build benchmark-queries-test`

## Design Considerations

### Data

Since there aren't timestamps on the data, I made the decision to move to the `TIMESTAMP` type from the `TIMESTAMPTZ` type.
I'm making the assumption that these have been all converted to the same timezone, UTC.  
With production data, we'd need to make sure that we handle timezones correctly.

### Indexes

Adding indexes in varying complexity to the `cpu_usage` table allow us to lookup data faster.
Making `cpu_usage` a hypertable as we configured it automatically makes an index on `ts`.

Using this example query:

```sql
explain
select max(usage) as max, min(usage) as min, date_trunc('minute', ts) as minute
from cpu_usage
where host = 'host_000003'
    and ts >= '2017-01-01 08:52:14'::timestamp
    and ts < '2017-01-01 09:52:14'::timestamp
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

Adding an index using `date_trunc('minute', ts))` or `host`, or other compound indexes using `ts` or truncated `ts` we get no change in the query plan. I am a bit surprised by this, but it seems to use the raw ts index is used well, and the size of the data leftover is not significant enough to merit usage of additional structures. Nothing I tried changed the query plan beyond the above or significantly affected benchmarks. In a scenario with different scale, compound indexes may play more of an effect here.
