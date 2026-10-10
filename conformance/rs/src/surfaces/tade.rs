//! tade — Typed Annotated Data Exchange: the framed byte stream every
//! stack CLI speaks. Encodings of each chunk constructor, LEB128 varints,
//! and the decode → molecule → re-encode mechanism.

use tade::{Chunk, ChunkId, Molecule, Reader, ReadResult, bytes};

use super::{Bytes, Context, Out};

fn chunks() -> Vec<(&'static str, Chunk)> {
    let nested = tade::encode_nested(&[Chunk::text("a"), Chunk::annotation("b")]);
    vec![
        ("text", Chunk::text("hello, tade")),
        ("text-empty", Chunk::text("")),
        ("annotation", Chunk::annotation("label")),
        ("error", Chunk::error("boom")),
        ("log", Chunk::log("info", "conformance", "message")),
        ("progress", Chunk::progress(7, "sync", 3, 10)),
        ("status", Chunk::status(-1)),
        ("scope", Chunk::scope(nested.clone())),
        ("compose", Chunk::compose(nested)),
        ("kv", tade::kv("key", Chunk::text("value"))),
        (
            "table",
            tade::table_chunk(&["a", "b"], vec![vec![Chunk::text("1"), Chunk::text("2")], vec![Chunk::text("3"), Chunk::text("4")]]),
        ),
        ("with-id", Chunk::text("x").with_id(ChunkId(9))),
        ("raw-300", Chunk::new(tade::sigil::BUC, tade::render::BINARY, bytes::Bytes::from(vec![0xA5u8; 300]))),
    ]
}

/// Run the tade surface.
pub fn run(_ctx: &Context, out: &mut Out) {
    let all = chunks();
    for (name, c) in &all {
        out.encoding(&format!("tade::Chunk::{name}@v1"), &c.encode());
    }

    let mut b = Bytes::new();
    for n in [0u64, 1, 127, 128, 255, 300, 16_383, 16_384, 1 << 32, u64::MAX] {
        let mut v = Vec::new();
        tade::encode_varint(n, &mut v);
        b.lv(&v);
    }
    out.encoding("tade::varint@v1", &b.0);

    // stream: all chunks back to back, read with the incremental reader in
    // 7-byte feeds, then each chunk through Molecule and re-encoded
    let mut stream = Vec::new();
    for (_, c) in &all {
        stream.extend_from_slice(&c.encode());
    }
    let mut reader = Reader::new();
    let mut decoded = Vec::new();
    for piece in stream.chunks(7) {
        reader.feed(piece);
        while let ReadResult::Chunk(c) = reader.next_chunk() {
            decoded.push(c);
        }
    }
    let mut b = Bytes::new();
    b.u64(decoded.len() as u64);
    for c in &decoded {
        b.lv(&c.encode());
        b.lv(&Molecule::from_chunk(c).to_chunk().encode());
    }
    out.mechanism("tade::Reader+Molecule@v1", "all-constructors/7-byte-feeds", &b.0);
    if decoded.len() != all.len() {
        out.fail(format!("tade reader decoded {} of {} chunks", decoded.len(), all.len()));
    }
}
