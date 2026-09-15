mod derive;
mod rpc;

pub use derive::{derive_from_seed, from_secret, validate_address, SuiSigner};
pub use rpc::SuiRpc;

pub const SUI_NATIVE: &str = "sui";
pub const SUI_COIN_TYPE: &str = "0x2::sui::SUI";
pub const SUI_DECIMALS: u8 = 9;
