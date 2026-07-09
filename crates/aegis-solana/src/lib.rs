mod keypair;
mod rpc;
mod transfer;

pub use keypair::{
    derive_keypair_from_mnemonic, generate_mnemonic, keypair_from_base64, keypair_to_base64,
    validate_mnemonic,
};
pub use rpc::SolanaRpc;
pub use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / solana_native_token::LAMPORTS_PER_SOL as f64
}
