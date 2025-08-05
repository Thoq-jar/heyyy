use clap::Parser;
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

#[derive(Parser)]
#[command(name = "heyyy")]
#[command(about = "A simple HTTP load tester")]
struct Args {
    #[arg(short = 'u', long = "url")]
    url: String,

    #[arg(short = 'c', long = "concurrency", default_value = "10")]
    req_per_sec: u64,

    #[arg(short = 'n', long = "requests", default_value = "100")]
    total_requests: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Args as Parser>::parse();

    println!("Starting load test:");
    println!("URL: {}", args.url);
    println!("Requests per second: {}", args.req_per_sec);
    println!("Total requests: {}", args.total_requests);
    println!();

    let client = Arc::new(Client::new());
    let semaphore = Arc::new(Semaphore::new(args.req_per_sec as usize));

    let start_time = Instant::now();
    let mut tasks = Vec::new();

    let delay_between_requests = Duration::from_nanos(1_000_000_000 / args.req_per_sec);

    for i in 0..args.total_requests {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let url = args.url.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            match client.get(&url).send().await {
                Ok(response) => {
                    let status = response.status();
                    (true, status.as_u16())
                }
                Err(_) => (false, 0),
            }
        });

        tasks.push(task);

        if i < args.total_requests - 1 {
            tokio::time::sleep(delay_between_requests).await;
        }
    }

    let mut successful_requests = 0;
    let mut failed_requests = 0;
    let mut status_codes = std::collections::HashMap::new();

    for task in tasks {
        match task.await {
            Ok((success, status_code)) => {
                if success {
                    successful_requests += 1;
                    *status_codes.entry(status_code).or_insert(0) += 1;
                } else {
                    failed_requests += 1;
                }
            }
            Err(_) => failed_requests += 1,
        }
    }

    let total_time = start_time.elapsed();
    let actual_req_per_sec = args.total_requests as f64 / total_time.as_secs_f64();

    println!("Test completed!");
    println!("Total time: {:.2} seconds", total_time.as_secs_f64());
    println!("Successful requests: {}", successful_requests);
    println!("Failed requests: {}", failed_requests);
    println!("Requests per second: {:.2}", actual_req_per_sec);

    if !status_codes.is_empty() {
        println!("\nStatus code distribution:");
        for (code, count) in status_codes {
            println!("  {}: {}", code, count);
        }
    }

    Ok(())
}