use aegis_solana::{normalize_mint, resolve_mint, ui_amount_to_raw};
use models::{SwapQuote, SwapResult, TokenInfo};

use crate::session::WalletService;
use crate::WalletError;

impl WalletService {
    pub async fn resolve_token(&self, mint: &str) -> Result<TokenInfo, WalletError> {
        let mint = normalize_mint(mint).map_err(WalletError::Operation)?;
        resolve_mint(&mint).await.map_err(WalletError::Operation)
    }

    pub async fn preview_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapQuote, WalletError> {
        let _ = self.require_pubkey()?;
        let input_mint = normalize_mint(input_mint).map_err(WalletError::Operation)?;
        let output_mint = normalize_mint(output_mint).map_err(WalletError::Operation)?;
        let amount_raw = ui_amount_to_raw(&input_mint, amount_ui)
            .await
            .map_err(WalletError::Operation)?;
        self.rpc_handle()
            .quote_swap(&input_mint, &output_mint, amount_raw, slippage_bps)
            .await
            .map_err(WalletError::Operation)
    }

    pub async fn execute_swap(
        &self,
        password: &str,
        input_mint: &str,
        output_mint: &str,
        amount_ui: f64,
        slippage_bps: u16,
    ) -> Result<SwapResult, WalletError> {
        self.verify_password(password)?;
        let keypair = self.signing_keypair()?;
        let input_mint = normalize_mint(input_mint).map_err(WalletError::Operation)?;
        let output_mint = normalize_mint(output_mint).map_err(WalletError::Operation)?;
        let amount_raw = ui_amount_to_raw(&input_mint, amount_ui)
            .await
            .map_err(WalletError::Operation)?;
        self.rpc_handle()
            .execute_swap(&keypair, &input_mint, &output_mint, amount_raw, slippage_bps)
            .await
            .map_err(WalletError::Operation)
    }
}
