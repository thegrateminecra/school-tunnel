use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

#[derive(Parser)]
#[command(
    name = "tunnel-client",
    about = "School laptop client - routes traffic through home PC"
)]
struct Args {
    /// Home server address (e.g. 1.2.3.4:443 or myhost.ddns.net:443)
    #[arg(short, long)]
    server: String,

    /// Authentication token (must match server)
    #[arg(short, long)]
    token: String,

    /// Local SOCKS5 port to listen on
    #[arg(short, long, default_value = "1080")]
    local_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let server_url = if args.server.contains("://") {
        args.server.clone()
    } else {
        format!("wss://{}", args.server)
    };

    let addr: SocketAddr = format!("127.0.0.1:{}", args.local_port)
        .parse()
        .context("invalid local port")?;

    let listener = TcpListener::bind(addr).await?;

    println!("========================================");
    println!("  Tunnel Client Running");
    println!("  Local SOCKS5: 127.0.0.1:{}", args.local_port);
    println!("  Tunneling to: {}", args.server);
    println!("  Browser proxy settings:");
    println!("    SOCKS Host: 127.0.0.1");
    println!("    SOCKS Port: {}", args.local_port);
    println!("    [x] Proxy DNS when using SOCKS v5");
    println!("========================================");

    let server_url = std::sync::Arc::new(server_url);
    let token = std::sync::Arc::new(args.token);

    loop {
        let (stream, addr) = listener.accept().await?;
        log::info!("Local connection from {}", addr);

        let url = server_url.clone();
        let tok = token.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_socks5(stream, url, tok).await {
                log::error!("SOCKS5 error from {}: {:?}", addr, e);
            }
        });
    }
}

async fn parse_socks5_connect(stream: &mut tokio::net::TcpStream) -> Result<String> {
    let mut buf = [0u8; 512];

    // SOCKS5 greeting
    stream
        .read_exact(&mut buf[..2])
        .await
        .context("reading SOCKS5 greeting")?;
    if buf[0] != 0x05 {
        anyhow::bail!("not SOCKS5");
    }
    let n_methods = buf[1] as usize;
    stream
        .read_exact(&mut buf[..n_methods])
        .await
        .context("reading methods")?;

    // Reply: no auth
    stream
        .write_all(&[0x05, 0x00])
        .await
        .context("sending method response")?;

    // Request
    stream
        .read_exact(&mut buf[..4])
        .await
        .context("reading request header")?;
    let cmd = buf[1];
    let atyp = buf[3];

    if cmd != 0x01 {
        send_socks5_reply(stream, 0x07).await?;
        anyhow::bail!("unsupported cmd: {}", cmd);
    }

    let (host, port) = match atyp {
        0x01 => {
            stream
                .read_exact(&mut buf[..6])
                .await
                .context("reading IPv4")?;
            let ip = format!("{}.{}.{}.{}", buf[0], buf[1], buf[2], buf[3]);
            let port = u16::from_be_bytes([buf[4], buf[5]]);
            (ip, port)
        }
        0x03 => {
            stream.read_exact(&mut buf[..1]).await?;
            let len = buf[0] as usize;
            stream.read_exact(&mut buf[..len + 2]).await?;
            let domain = String::from_utf8_lossy(&buf[..len]).to_string();
            let port = u16::from_be_bytes([buf[len], buf[len + 1]]);
            (domain, port)
        }
        0x04 => {
            stream.read_exact(&mut buf[..18]).await?;
            let ipv6 = std::net::Ipv6Addr::from([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
                buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
            ]);
            let port = u16::from_be_bytes([buf[16], buf[17]]);
            (ipv6.to_string(), port)
        }
        _ => {
            send_socks5_reply(stream, 0x08).await?;
            anyhow::bail!("unsupported atyp: {}", atyp);
        }
    };

    Ok(format!("{}:{}", host, port))
}

async fn send_socks5_reply(stream: &mut tokio::net::TcpStream, rep: u8) -> Result<()> {
    stream
        .write_all(&[0x05, rep, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .await?;
    Ok(())
}

async fn handle_socks5(
    mut local: tokio::net::TcpStream,
    server_url: std::sync::Arc<String>,
    token: std::sync::Arc<String>,
) -> Result<()> {
    let target = parse_socks5_connect(&mut local).await?;
    log::info!("CONNECT -> {}", target);

    // Connect to tunnel server via WebSocket
    let mut request = (*server_url)
        .as_str()
        .into_client_request()
        .context("creating request")?;
    request.headers_mut().insert(
        "X-Auth-Token",
        token
            .parse()
            .context("invalid token")?,
    );

    let (ws_stream, _) = tokio_tungstenite::connect_async_tls_with_config(
        request,
        None,
        true, // accept invalid certs (ContentKeeper MITM)
        None, // default connector
    )
    .await
    .context("WebSocket connect")?;

    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Handshake
    let handshake = serde_json::json!({
        "token": *token,
        "target": target,
    });
    ws_write
        .send(Message::Text(handshake.to_string().into()))
        .await
        .context("sending handshake")?;

    let msg = ws_read
        .next()
        .await
        .context("no server response")??;
    let resp: serde_json::Value =
        serde_json::from_str(msg.to_text()?).context("bad server response")?;

    if resp["status"].as_str() != Some("ok") {
        let err = resp["msg"].as_str().unwrap_or("unknown");
        send_socks5_reply(&mut local, 0x05).await?;
        anyhow::bail!("server: {}", err);
    }

    send_socks5_reply(&mut local, 0x00).await?;
    log::info!("Tunnel active: {}", target);

    let (mut local_read, mut local_write) = tokio::io::split(local);

    // Channel for all WS writes
    let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let ws_writer = async move {
        while let Some(msg) = ws_rx.recv().await {
            if ws_write.send(msg).await.is_err() {
                break;
            }
        }
    };

    let ws_to_local = async {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    if local_write.write_all(&data).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Ping(data)) => {
                    let _ = ws_tx.send(Message::Pong(data));
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
        let _ = local_write.shutdown().await;
    };

    let local_to_ws = async {
        let mut buf = [0u8; 16384];
        loop {
            match local_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if ws_tx.send(Message::Binary(buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = ws_to_local => {}
        _ = local_to_ws => {}
        _ = ws_writer => {}
    }

    Ok(())
}
