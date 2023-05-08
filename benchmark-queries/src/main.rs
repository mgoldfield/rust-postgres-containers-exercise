use chrono::{DateTime, Utc};
use postgres::{Client, NoTls};
use threadpool::ThreadPool;

// TODO - handle unwraps

// in a production project, these values might get split out and put into config(s)
static POSTGRES_URL: &'static str = "postgresql://postgres:password@timescaledb:5432/homework";
static NUM_THREADS: usize = 4;

fn main() {
    let mut client: Client = ts_client();

    let cpus_to_measure = client
        .query("select distinct hostname from cpu_stats_queries", &[])
        .unwrap();

    let pool = ThreadPool::new(NUM_THREADS);

    for cpu_row in cpus_to_measure {
        let cpu = String::from(cpu_row.get::<usize, &str>(0));
        pool.execute(move || get_stats_for_cpu(cpu));
    }
    pool.join();
}

fn get_stats_for_cpu(cpu: String) {
    let mut client = ts_client();
    let cpu_queries = client
        .query(
            "select start_time, end_time 
                from cpu_stats_queries where hostname = $1",
            &[&cpu],
        )
        .unwrap();

    for ranges in cpu_queries {
        let start_time: DateTime<Utc> = ranges.get(0);
        let end_time: DateTime<Utc> = ranges.get(1);
        let cpu_stats = client
            .query(
                "select max(usage) as max, min(usage) as min, minute
                from (
                    select date_trunc('minute', ts) as minute, usage
                    from cpu_usage 
                    where host = $1
                        and ts >= $2 and ts < $3) as stats_for_host
                group by minute",
                &[&cpu, &start_time, &end_time],
            )
            .unwrap();

        let max: f64 = cpu_stats[0].get(0);
        let min: f64 = cpu_stats[0].get(1);
        let minute: DateTime<Utc> = cpu_stats[0].get(2);

        println!(
            "host: {}, max: {}, min: {}, minute: {}",
            cpu, max, min, minute
        );
    }
}

fn ts_client() -> Client {
    return Client::connect(POSTGRES_URL, NoTls).unwrap();
}

#[cfg(test)]
mod test {
    use crate::ts_client;

    #[test]
    fn timescale_is_up_and_data_loaded() {
        let mut client = ts_client();
        let cpu_stats_queries_rows: i64 = client
            .query("select count(1) from cpu_stats_queries", &[])
            .unwrap()[0]
            .get(0);
        assert_eq!(cpu_stats_queries_rows, 200);

        let cpu_usage_rows: i64 =
            client.query("select count(1) from cpu_usage", &[]).unwrap()[0].get(0);
        assert_eq!(cpu_usage_rows, 345600);
    }
}
