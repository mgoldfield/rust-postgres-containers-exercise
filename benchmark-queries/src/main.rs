use chrono::NaiveDateTime;
use devtimer::DevTime;
use postgres::{Client, NoTls};
use std::cmp::Ordering;
use std::fmt;
use std::sync::mpsc::{channel, Receiver, Sender};
use threadpool::ThreadPool;

// in a production project, the following values might get split out and put into config(s)
static POSTGRES_URL: &'static str = "postgresql://postgres:password@timescaledb:5432/homework";
// this can be tuned depending on the system.  max_connections on the database should be >= NUM_THREADS
static NUM_THREADS: usize = 4;

fn main() {
    let mut client: Client = ts_client();

    let cpus_to_measure = client
        .query("select distinct hostname from cpu_stats_queries", &[])
        .expect("query for hosts to measure failed");

    let pool = ThreadPool::new(NUM_THREADS);
    let (sender, reciever) = channel::<CpuQueryBenchmark>();

    for cpu_row in cpus_to_measure {
        let sender = sender.clone();
        let cpu = String::from(cpu_row.get::<usize, &str>(0));
        pool.execute(move || get_stats_for_cpu(cpu, sender));
    }
    pool.join();
    drop(sender);
    compute_stats(reciever);
}

fn get_stats_for_cpu(cpu: String, sender: Sender<CpuQueryBenchmark>) {
    let mut client = ts_client();
    let cpu_queries = client
        .query(
            "select start_time, end_time 
                from cpu_stats_queries where hostname = $1",
            &[&cpu],
        )
        .expect(format!("query for periods for host {} failed", cpu).as_str());

    for ranges in cpu_queries {
        let cq = CpuQuery {
            start_time: ranges.get(0),
            end_time: ranges.get(1),
            host: cpu.clone(),
        };

        let mut timer = DevTime::new_simple();
        timer.start();
        let cpu_stats_wrapped = client.query(
            "select max(usage) as max, min(usage) as min, date_trunc('minute', ts) as minute
            from cpu_usage
            where host = $1
                and ts >= $2
                and ts < $3
            group by date_trunc('minute', ts);",
            &[&cq.host, &cq.start_time, &cq.end_time],
        );
        timer.stop();

        let cpu_stats = cpu_stats_wrapped.expect(format!("query for {} failed", cq).as_str());

        println!(
            "host: {}, max: {}, min: {}, minute: {}",
            cq.host,
            cpu_stats[0].get::<usize, f64>(0),
            cpu_stats[0].get::<usize, f64>(1),
            cpu_stats[0].get::<usize, NaiveDateTime>(2)
        );

        sender
            .send(CpuQueryBenchmark {
                execute_time: timer.time_in_millis().expect("timer failed"),
                cq,
            })
            .expect("sending stats to main thread failed");
    }
}

fn compute_stats(receiver: Receiver<CpuQueryBenchmark>) {
    let mut query_data: Vec<CpuQueryBenchmark> = receiver.iter().collect();

    let query_times: Vec<f64> = query_data
        .clone()
        .into_iter()
        .map(|c| c.execute_time as f64)
        .collect();

    query_data.sort_by(|a, b| {
        if a.execute_time >= b.execute_time {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    let mean = statistical::mean(&query_times);
    let median = statistical::median(&query_times);
    let std_dev = statistical::standard_deviation(&query_times, Some(mean));

    println!(
        "
        mean: {}, median: {}, standard deviation: {}
        min and max queries:
            {}
            {}",
        mean,
        median,
        std_dev,
        query_data[0],
        query_data.last().expect("no data recieved from threads")
    );
}

fn ts_client() -> Client {
    return Client::connect(POSTGRES_URL, NoTls).expect("connection to database failed");
}

#[derive(Clone)]
struct CpuQuery {
    host: String,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
}

#[derive(Clone)]
struct CpuQueryBenchmark {
    cq: CpuQuery,
    execute_time: u128,
}

impl fmt::Display for CpuQuery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "host: {}, start_time: {}, end_time: {}",
            self.host, self.start_time, self.end_time
        )
    }
}

impl fmt::Display for CpuQueryBenchmark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "query time: {}ms {}", self.execute_time, self.cq)
    }
}

#[cfg(test)]
mod test {
    use crate::ts_client;

    // with more time I would like to have fleshed this section out more, but leaving it this way in the interest of time

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
