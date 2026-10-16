//! QUIC through radio — chain-wire.md §1: one endpoint per node, ALPN
//! `cyber/chain/1`, the node key in `$home/<chain>/radio.key`, tade frames
//! on every stream with the dialect declared first, sizes bounded before a
//! byte is allocated.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use radio::endpoint::{Connection, RecvStream, SendStream};
use radio::{Endpoint, RelayMode, SecretKey};

use super::frames::{self, Frame, FrameError, DIALECT, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES};

pub const ALPN: &[u8] = b"cyber/chain/1";
pub const STREAM_TIMEOUT: Duration = Duration::from_secs(30);
/// Frames per response: headers (1) + end, blocks ≤ 128 + end, txs ≤ 256 + end.
pub const MAX_RESPONSE_FRAMES: usize = 257;

#[derive(Debug)]
pub enum WireError {
    Io(String),
    Frame(FrameError),
    /// The first frame was not the dialect declaration.
    Dialect,
    Timeout,
    /// The peer answered `x`.
    Remote { code: u8, reason: String },
    Protocol(&'static str),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::Io(e) => write!(f, "io: {e}"),
            WireError::Frame(e) => write!(f, "frame: {e}"),
            WireError::Dialect => write!(f, "stream did not declare the chain dialect"),
            WireError::Timeout => write!(f, "stream timed out"),
            WireError::Remote { code, reason } => write!(f, "peer error {code}: {reason}"),
            WireError::Protocol(what) => write!(f, "protocol: {what}"),
        }
    }
}

impl WireError {
    /// chain-wire §5: a frame over size is +100; malformed frames +10.
    pub fn score(&self) -> u32 {
        match self {
            WireError::Frame(FrameError::Oversize(_)) => 100,
            WireError::Frame(_) | WireError::Dialect | WireError::Protocol(_) => 10,
            WireError::Io(_) | WireError::Timeout | WireError::Remote { .. } => 0,
        }
    }
}

impl From<FrameError> for WireError {
    fn from(e: FrameError) -> Self {
        WireError::Frame(e)
    }
}

/// Load the node key, generating it on first start (32 bytes, hex, mode 0600).
pub fn load_or_create_key(path: &Path) -> std::io::Result<SecretKey> {
    if let Ok(text) = std::fs::read_to_string(path) {
        let bytes = super::codec::unhex32(text.trim()).ok_or_else(|| std::io::Error::other("radio.key is not 64 hex chars"))?;
        return Ok(SecretKey::from_bytes(&bytes));
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let key = SecretKey::generate(&mut rand::rng());
    std::fs::write(path, super::codec::hex(&key.to_bytes()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(key)
}

/// Bind the endpoint: direct UDP at `bind` (or an ephemeral port), relays
/// only when `relay` is set (nodes behind NAT), no DNS discovery — peers
/// come from the bootstrap file and the graph.
pub async fn bind(key: SecretKey, bind: Option<SocketAddr>, relay: bool, alpns: Vec<Vec<u8>>) -> Result<Endpoint, String> {
    let mode = if relay { RelayMode::Default } else { RelayMode::Disabled };
    let mut builder = Endpoint::empty_builder(mode).secret_key(key).alpns(alpns);
    if let Some(addr) = bind {
        builder = builder.clear_ip_transports().bind_addr(addr).map_err(|e| e.to_string())?;
    }
    builder.bind().await.map_err(|e| e.to_string())
}

/// Write one frame.
pub async fn write_frame(send: &mut SendStream, frame: &Frame) -> Result<(), WireError> {
    send.write_all(&frame.encode()).await.map_err(|e| WireError::Io(e.to_string()))
}

/// Read one frame with its payload bounded by `max` before allocation.
pub async fn read_frame(recv: &mut RecvStream, max: usize) -> Result<Frame, WireError> {
    let mut head = [0u8; 3];
    recv.read_exact(&mut head).await.map_err(|e| WireError::Io(e.to_string()))?;
    if head[0] != tade::MARKER {
        return Err(FrameError::Marker.into());
    }
    let mut varint = Vec::with_capacity(10);
    let len = loop {
        let mut b = [0u8; 1];
        recv.read_exact(&mut b).await.map_err(|e| WireError::Io(e.to_string()))?;
        varint.push(b[0]);
        match frames::varint(&varint) {
            Ok((len, _)) => break len,
            Err(FrameError::Truncated) if varint.len() < 10 => continue,
            Err(e) => return Err(e.into()),
        }
    };
    if len > max as u64 {
        return Err(FrameError::Oversize(len as usize).into());
    }
    let mut payload = vec![0u8; len as usize];
    recv.read_exact(&mut payload).await.map_err(|e| WireError::Io(e.to_string()))?;
    Ok(Frame::from_parts(head[1], head[2], &payload)?)
}

/// Read the dialect declaration that must open every stream.
pub async fn read_dialect(recv: &mut RecvStream) -> Result<(), WireError> {
    match tokio::time::timeout(STREAM_TIMEOUT, read_frame(recv, 128)).await {
        Ok(Ok(Frame::Dialect(urn))) if urn == DIALECT => Ok(()),
        Ok(Ok(_)) => Err(WireError::Dialect),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(WireError::Timeout),
    }
}

/// One request on a fresh bidirectional stream: dialect, request, then the
/// response frames up to `e` (or `x`), each bounded; 30 s for the whole exchange.
pub async fn request(conn: &Connection, req: Frame) -> Result<Vec<Frame>, WireError> {
    tokio::time::timeout(STREAM_TIMEOUT, async {
        let (mut send, mut recv) = conn.open_bi().await.map_err(|e| WireError::Io(e.to_string()))?;
        write_frame(&mut send, &Frame::Dialect(DIALECT.into())).await?;
        write_frame(&mut send, &req).await?;
        send.finish().map_err(|e| WireError::Io(e.to_string()))?;
        // the responder's side of the stream opens with the dialect too
        match read_frame(&mut recv, 128).await? {
            Frame::Dialect(urn) if urn == DIALECT => {}
            _ => return Err(WireError::Dialect),
        }
        let mut out = Vec::new();
        loop {
            match read_frame(&mut recv, MAX_RESPONSE_BYTES).await? {
                Frame::End => return Ok(out),
                Frame::Error { code, reason } => return Err(WireError::Remote { code, reason }),
                frame if matches!(frame, Frame::Dialect(_)) || (frame.is_request() && !matches!(frame, Frame::Status(_))) => {
                    return Err(WireError::Protocol("request frame in a response"))
                }
                frame => {
                    out.push(frame);
                    if out.len() > MAX_RESPONSE_FRAMES {
                        return Err(WireError::Protocol("too many response frames"));
                    }
                }
            }
        }
    })
    .await
    .unwrap_or(Err(WireError::Timeout))
}

/// Read the request that opens an accepted stream: dialect then one request frame.
pub async fn read_request(recv: &mut RecvStream) -> Result<Frame, WireError> {
    read_dialect(recv).await?;
    match tokio::time::timeout(STREAM_TIMEOUT, read_frame(recv, MAX_REQUEST_BYTES)).await {
        Ok(Ok(frame)) if frame.is_request() => Ok(frame),
        Ok(Ok(_)) => Err(WireError::Protocol("not a request frame")),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(WireError::Timeout),
    }
}
