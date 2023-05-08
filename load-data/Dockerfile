FROM timescale/timescaledb:latest-pg15

# these paths are in reference to docker-compose.yaml
COPY ./load-data/cpu_usage.* ./
COPY ./load-data/query_params.csv ./ 
COPY ./load-data/create-databases-and-copy-data.sh ./ 

CMD ./create-databases-and-copy-data.sh