mod jupiter;
mod keypair;
mod price;
mod rpc;
mod swap;
mod token_metadata;
mod transfer;

pub use jupiter::{configure_jupiter_api_key, shorten_mint, WRAPPED_SOL_MINT};
pub use keypair::{
    derive_keypair_from_mnemonic, generate_mnemonic, keypair_from_base64, keypair_to_base64,
    validate_mnemonic,
};
pub use models::MANAGED_DEFAULT_RPC_URL;
pub use price::{get_prices, get_sol_price};
pub use rpc::SolanaRpc;
pub use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
pub use swap::{normalize_mint, ui_amount_to_raw};
pub use token_metadata::{
    curated_major_mints, get_metadata, resolve_mint, resolve_mint_local, search_tokens,
};

pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / solana_native_token::LAMPORTS_PER_SOL as f64
}
