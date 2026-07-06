use wallet_core::WalletService;

pub struct AppState {
    pub wallet: WalletService,
}

impl AppState {
    pub fn new(wallet: WalletService) -> Self {
        Self { wallet }
    }
}
