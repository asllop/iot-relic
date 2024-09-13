mod nr_api;

use clap::Parser;
use tokio::{
    net::UdpSocket,
    sync::mpsc
};
use std::{
    io, time,
};
use nrltp_server::{
    NrltpDatagram, HunkBody, IntMetricHunk
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Server host.
    #[clap(short, long, default_value = "0.0.0.0")]
    host: String,

    /// Server port.
    #[clap(short, long, default_value_t = 8888)]
    port: u16,

    /// New Relic License Key.
    #[clap(short, long)]
    license_key: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, mut rx) = mpsc::channel(1000);

    let args = Args::parse();
    
    println!("Binding {}:{} ...", args.host, args.port);

    let server_port = format!("{}:{}", args.host, args.port);
    let sock = UdpSocket::bind(server_port).await?;

    let mut buf = vec![0; 64*1024];

    println!("Starting main server loop...");

    tokio::spawn(async move {
        let mut metric_buffer = Vec::<IntMetricHunk>::new();
        let mut client_id = String::new();
        let mut last_sync_time = time::Instant::now();

        loop {
            let body = rx.recv().await.expect("Could not received message from UDP server thread");

            match body {
                HunkBody::ClientId(cid_hunk) => {
                    println!("Got a ClientId hunk");
                    client_id = cid_hunk.client_id().into();
                },
                HunkBody::IntMetric(intm_hunk) => {
                    println!("Got an IntMetric hunk");
                    metric_buffer.push(intm_hunk);
                },
                _ => {
                    println!("Got something else hunk");
                }
            }
            
            if metric_buffer.len() > 100 || last_sync_time.elapsed().as_secs() > 10 {
                if let Err(e) = nr_api::nr_push_metrics(&args.license_key, &client_id, &mut metric_buffer).await {
                    println!("Error pushing metrics to New Relic: {}\n", e);
                }
                last_sync_time = time::Instant::now();
                metric_buffer.clear();
            }
        }
    });

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, addr);

        let datagram = NrltpDatagram::new(&buf, len);
        for hunk in datagram {
            tx.send(hunk.body()).await.expect("Could not send message to NR Push thread");
        }

        println!("\n");
    }
}