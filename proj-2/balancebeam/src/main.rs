mod request;
mod response;

use clap::Parser;
use rand::{Rng, SeedableRng};
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::time::Duration;

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[clap(about = "Fun with load balancing")]
struct CmdOptions {
    /// "IP/port to bind to"
    #[arg(short, long, default_value = "0.0.0.0:1100")]
    bind: String,
    /// "Upstream host to forward requests to"
    #[arg(short, long)]
    upstream: Vec<String>,
    /// "Perform active health checks on this interval (in seconds)"
    #[arg(long, default_value = "10")]
    active_health_check_interval: usize,
    /// "Path to send request to for active health checks"
    #[arg(long, default_value = "/")]
    active_health_check_path: String,
    /// "Maximum number of requests to accept per IP per minute (0 = unlimited)"
    #[arg(long, default_value = "0")]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    #[allow(dead_code)]
    active_health_check_interval: usize,
    /// Where we should send requests when doing active health checks (Milestone 4)
    #[allow(dead_code)]
    active_health_check_path: String,
    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    #[allow(dead_code)]
    max_requests_per_minute: usize,
    /// Total number of requests in a minute
    total_requests_in_a_minute: Arc<Mutex<usize>>,
    /// Addresses of servers that we are proxying to
    #[allow(dead_code)]
    upstream_addresses: Vec<String>,
    /// Addresses of servers that are alive
    live_upstream_addresses: Arc<RwLock<Vec<String>>>,
}

#[tokio::main]
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // Start listening for connections
    let listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    // Handle incoming connections
    let state = Arc::new(ProxyState {
        live_upstream_addresses: Arc::new(RwLock::new(options.upstream.clone())),
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        total_requests_in_a_minute: Arc::new(Mutex::new(0))
    });

    // health check
    health_check(state.clone());

    // reset fixed window
    reset_minute_requests(state.clone());

    // Listen
    while let Ok((stream, _)) = listener.accept().await {
        let state_clone = state.clone();
        // Handle the connection!
        tokio::spawn(async move {
            // 先 move 进 closure，再借用
            handle_connection(stream, &state_clone).await;
        });
    }
}

fn health_check(state: Arc<ProxyState>) {
    tokio::spawn(async move {
        let interval = state.active_health_check_interval;
        let check_path = state.active_health_check_path.clone();
        loop {
            tokio::time::sleep(Duration::from_secs(interval as u64)).await;

            let mut threads = vec![];
            for upstream_ip in state.upstream_addresses.iter() {
                let upstream_ip = upstream_ip.clone();
                let check_path_clone = check_path.clone();
                threads.push((upstream_ip.clone(), tokio::spawn(async move {
                    check_upstream(upstream_ip.clone(), check_path_clone).await
                })));
            }

            for (upstream_ip, handle) in threads {
                match handle.await {
                    Ok(join_result) => match join_result {
                        Ok(_) => {
                            log::debug!("Add {} to live upstream addresses", upstream_ip);
                            add_to_live_upstream_address(&state, upstream_ip).await;
                        }
                        Err(_) => {
                            log::debug!("Remove {} from live upstream addresses", upstream_ip);
                            remove_from_live_upstream_address(&state, upstream_ip).await;
                        }
                    }
                    Err(_) => {
                        log::debug!("Remove {} from live upstream addresses", upstream_ip);
                        remove_from_live_upstream_address(&state, upstream_ip).await;
                    }
                }
            }
        }
    });
}

fn reset_minute_requests(state: Arc<ProxyState>) {
    tokio::spawn(async move {
        loop {
            // reset per minute
            tokio::time::sleep(Duration::from_secs(60)).await;

            // reset
            let mut times = state.total_requests_in_a_minute.lock().await;
            log::debug!("Reset times: {}", *times);
            *times = 0;
        }
    });
}

async fn check_upstream(upstream_ip: String, check_path: String) -> Result<(), std::io::Error> {
    // connect
    let mut upstream = TcpStream::connect(upstream_ip.clone()).await?;

    // request
    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(check_path)
        .header("Host", upstream_ip)
        .body(Vec::new()).unwrap();

    // check request
    request::write_to_stream(&request, &mut upstream).await?;

    // check response
    match response::read_from_stream(&mut upstream, request.method()).await {
        Ok(response) => if response.status().is_server_error() {
            Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "500"))
        } else {
            Ok(())
        }
        Err(_) => {
            Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "500"))
        }
    }
}

// 对 upstream_addresses 的访问是互斥呢还是读写呢？
// 访问之后才知道是不是 dead
// 不等待访问结果；得到结果之后再次操作
// 既然如此，那就用读写锁
async fn select_upstream_address_randomly(state: &Arc<ProxyState>) -> Option<String> {
    let live_upstream_addresses = state.live_upstream_addresses.read().await;
    if live_upstream_addresses.len() > 0 {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let upstream_idx = rng.gen_range(0..live_upstream_addresses.len());
    
        Some(live_upstream_addresses.get(upstream_idx)?.to_string())
    } else {
        None
    }
}

async fn remove_from_live_upstream_address(state: &Arc<ProxyState>, upstream_ip: String) -> Vec<String> {
    let mut live_upstream_addresses = state.live_upstream_addresses.write().await;

    let still_live_upstream_addresses: Vec<String> = live_upstream_addresses
        .iter()
        .filter(|other_upstream_ip| **other_upstream_ip != upstream_ip)
        .map(|upstream_ip| upstream_ip.to_owned())
        .collect();

    *live_upstream_addresses = still_live_upstream_addresses.clone();

    still_live_upstream_addresses
}

async fn add_to_live_upstream_address(state: &Arc<ProxyState>, upstream_ip: String) -> Vec<String> {
    let mut live_upstream_addresses = state.live_upstream_addresses.write().await;

    live_upstream_addresses.push(upstream_ip);

    (*live_upstream_addresses).clone()
}

async fn connect_to_upstream(state: &Arc<ProxyState>) -> Result<TcpStream, std::io::Error> {
    // implement failover (milestone 3)
    loop {
        let upstream_ip = match select_upstream_address_randomly(state).await {
            Some(upstream_ip) => upstream_ip,
            None => {
                return Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Failed to connect to upstream"));
            }
        };

        match TcpStream::connect(upstream_ip.clone()).await.or_else(|err| {
            log::error!("Failed to connect to upstream {}: {}", upstream_ip, err);
            Err(err)
        }) {
            Ok(stream) => {
                return Ok(stream);
            }
            Err(err) => {
                let still_live_upstream_addresses = remove_from_live_upstream_address(state, upstream_ip).await;
                if still_live_upstream_addresses.len() == 0 {
                    return Err(err);
                }
            }
        }
    }
}

async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("{} <- {}", client_ip, response::format_response_line(&response));
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

async fn handle_connection(mut client_conn: TcpStream, state: &Arc<ProxyState>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!("Connection received from {}", client_ip);

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
    };
    let upstream_ip = upstream_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );

        // check request rate
        let mut times = state.total_requests_in_a_minute.lock().await;
        if (*times) < state.max_requests_per_minute {
            *times += 1;
            log::debug!("Request Ok: {}", *times);
            drop(times);
        } else {
            log::debug!("Too many requests: {} >= {}", *times, state.max_requests_per_minute);
            drop(times);

            // too many request
            let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
            send_response(&mut client_conn, &response).await;

            continue;
        }

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!("Failed to send request to upstream {}: {}", upstream_ip, error);
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
