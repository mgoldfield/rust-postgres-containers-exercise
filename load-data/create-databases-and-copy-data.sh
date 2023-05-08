# TODO: sanitize data
# TODO: only run these commands if the infrastructure isnt present currently 

# echo "creating database infrastructure ...";
psql postgresql://postgres:password@timescaledb:5432 < cpu_usage.sql; 

# echo "populating data ...";
psql postgresql://postgres:password@timescaledb:5432/homework -c "\COPY cpu_usage FROM ./cpu_usage.csv CSV HEADER";
psql postgresql://postgres:password@timescaledb:5432/homework -c "\COPY cpu_stats_queries FROM ./query_params.csv CSV HEADER"; 
