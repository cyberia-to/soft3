mod domain; mod claim; mod cosmos;
use k256::ecdsa::SigningKey;
fn hex(b:&[u8])->String{b.iter().map(|b|format!("{b:02x}")).collect()}
fn main(){let mut rows=Vec::new();
for (domain,hrp,body) in [("example.com","lytics",b"{\"hello\":\"world\"}".as_slice()),("cyberia.my","cyber",b"from|rel|to|1.250000".as_slice())]{
let k=domain::DomainKey::derive(&[7;32],domain,hrp).unwrap();
rows.push(serde_json::json!({"entropy_hex":hex(&[7;32]),"domain":domain,"hrp":hrp,"body":String::from_utf8(body.to_vec()).unwrap(),"pubkey_hex":hex(&k.pubkey),"address":k.bech32,"native_hex":hex(&k.native),"signature_hex":hex(&claim::sign_arbitrary(k.signing_key(),&k.bech32,body)),"adr036_doc":claim::adr036_doc(&k.bech32,body)}));
}println!("{}",serde_json::to_string_pretty(&rows).unwrap());}
#[derive(Debug)]
pub enum Error {
    /// The BIP-39 mnemonic could not be parsed.
    Mnemonic(String),
    /// HD derivation (BIP-32/44) failed for the given path.
    Derive(String),
    /// Address encoding failed (bad HRP or bech32 error).
    Bech32(String),
    /// A key or signature was malformed.
    Key(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Mnemonic(m) => write!(f, "invalid mnemonic: {m}"),
            Error::Derive(m) => write!(f, "HD derivation failed: {m}"),
            Error::Bech32(m) => write!(f, "address encoding failed: {m}"),
            Error::Key(m) => write!(f, "malformed key or signature: {m}"),
        }
    }
}

impl std::error::Error for Error {}
