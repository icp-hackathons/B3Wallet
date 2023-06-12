use bitcoin::{PublicKey, Script, ScriptBuf};

/// The length of the transaction signature.
pub const MAX_ENCODED_SIGNATURE_LEN: usize = 73;

pub const PUBKEY_LEN: usize = 32;

const MOCK_SIG: [u8; MAX_ENCODED_SIGNATURE_LEN] = [
    0x30, 70, 0x02, 33, 0x00, 0x8f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 33, 0x00, 0x8f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

pub fn mock_signer(public_key: &PublicKey) -> ScriptBuf {
    // Add signature and public key to script
    let script = Script::builder()
        .push_slice(&MOCK_SIG)
        .push_key(&public_key)
        .into_script();

    script
}