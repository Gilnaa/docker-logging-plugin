use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    convert::Infallible,
    io::IoSlice,
};

use protobuf::Message;
use tokio::{task::JoinHandle, io::{AsyncReadExt, AsyncWriteExt}};
use tokio::sync::Mutex;
use tokio::fs::File;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use hyperlocal::UnixServerExt;

mod generated_protos;
mod docker_plugin_api;

use generated_protos::entry::LogEntry;


#[derive(Debug)]
struct LoggerTaskContext {
    join_handle: JoinHandle<()>,
}

type RunningLoggersMap = Arc<Mutex<HashMap<String, LoggerTaskContext>>>;

async fn streamer_task_main(mut fifo: File, mut output_file: File) -> Result<(), String> {
    let mut buffer = Vec::<u8>::new();
    loop {
        buffer.clear();

        let size = fifo.read_u32().await.map_err(|e| {
            format!("Failed reading message length {e:}")
        })?;

        buffer.reserve(size as usize);
        (&mut fifo).take(size as u64).read_to_end(&mut buffer).await.map_err(|e| {
            format!("Failed reading message payload with size {size}: {e:}")
        })?;

        let log_entry = LogEntry::parse_from_bytes(&buffer).map_err(|e| {
            format!("Failed parsing log message {e:}")
        })?;

        let line_header = format!("[{}:{}] ", log_entry.time_nano, log_entry.source);
        output_file.write_vectored(&[
            IoSlice::new(line_header.as_bytes()),
            IoSlice::new(&log_entry.line),
            IoSlice::new(b"\n"),
        ]).await.map_err(|e| {
            format!("Failed writing log to output {e:}")
        })?;
    }
}

async fn start_streamer(request: &docker_plugin_api::StartLoggingRequest) -> Result<LoggerTaskContext, String> {
    eprintln!("Starting streamer for request: {request:?}");
    let container_id_or_name_maybe = request.info.container_name.as_deref().unwrap_or("unnamed");
    let container_id_or_name_maybe = container_id_or_name_maybe.replace('/', "_");
    let file_name = format!("/core/container_logs_{container_id_or_name_maybe}_{}", request.info.container_id);
    
    let fifo = File::open(&request.file).await.map_err(|err| {
        format!("Failed opening FIFO: {err:?}")
    })?;

    let output_file = std::fs::OpenOptions::new().create(true).append(true).open(&file_name).map_err(|err| {
        format!("Failed opening output file: {file_name} {err:?}")
    })?;
    let output_file = File::from_std(output_file);

    Ok(
        LoggerTaskContext {
            join_handle: tokio::spawn(async {
                let result = streamer_task_main(fifo, output_file).await;
                eprintln!("Log streamer finished with {result:?}");
            }),
        }
    )
}

async fn handle_request(req: Request<Body>, clients_map: RunningLoggersMap) -> Result<(), String> {
    let uri = req.uri().clone();
    eprintln!("{:?} path={}", req, uri.path());

    let body = hyper::body::to_bytes(req.into_body()).await.map_err(|err| {
        format!("Failed reading request body {err:?}")
    })?;

    match uri.path() {
        "/LogDriver.StartLogging" => {
            let request = serde_json::from_slice::<docker_plugin_api::StartLoggingRequest>(&body).map_err(|err| {
                eprintln!("Failed parsing request: {err}");
                String::from("Failed parsing request")
            })?;

            let client = start_streamer(&request).await?;
            clients_map.lock().await.insert(request.file, client);
        },
        "/LogDriver.StopLogging" => {
            let request = serde_json::from_slice::<docker_plugin_api::StopLoggingRequest>(&body).map_err(|err| {
                eprintln!("Failed parsing request: {err}");
                String::from("Failed parsing request")
            })?;

            let file = clients_map.lock().await.remove(&request.file).ok_or_else(|| {
                format!("Failed finding logger for file {}", request.file)
            })?;

            file.join_handle.abort();
        },
        unknown_endpoint => {
            return Err(format!("Unrecognized endpoint {unknown_endpoint}"));
        }
    }
    Ok(())
}

async fn handle_request_wrapper(req: Request<Body>, clients_map: RunningLoggersMap) -> Result<Response<String>, Infallible> {
    match handle_request(req, clients_map).await {
        Ok(()) => Ok(Response::new("{}".into())),
        Err(error_string) => {
            let response = Response::builder().status(500).body(
                format!("{{ \"Err\": \"{error_string}\"}}")
            ).unwrap();
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() {
    eprintln!("Starting drivents logger plugin");

    let socket_path = Path::new("/run/docker/plugins/logger.sock");
    if socket_path.exists() {
        std::fs::remove_file(socket_path).unwrap();
    }
    let client_map: RunningLoggersMap = Arc::new(Mutex::new(HashMap::new()));

    let make_service = make_service_fn(move |_conn| {
        let client_map = client_map.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| handle_request_wrapper(req, client_map.clone())))
        }
    });

    let server = Server::bind_unix(socket_path).unwrap();
    server.serve(make_service).await.unwrap();
}
