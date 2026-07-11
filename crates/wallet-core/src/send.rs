use models::{SendPreview, SendResult};

use crate::session::WalletService;
use crate::WalletError;

impl WalletService {
    pub async fn preview_sol_send(
        &self,
        to: &str,
        amount_sol: f64,
    ) -> Result<SendPreview, WalletError> {
        let keypair = self.signing_keypair()?;
        self.rpc_handle()
            .preview_sol_send(&keypair, to, amount_sol)
            .await
            .map_err(WalletError::Operation)
    }

    pub async fn preview_spl_send(
        &self,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendPreview, WalletError> {
        let keypair = self.signing_keypair()?;
        self.rpc_handle()
            .preview_spl_send(&keypair, mint, to, amount)
            .await
            .map_err(WalletError::Operation)
    }

    pub async fn send_sol(
        &self,
        password: &str,
        to: &str,
        amount_sol: f64,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        let keypair = self.signing_keypair()?;
        self.rpc_handle()
            .send_sol(&keypair, to, amount_sol)
            .await
            .map_err(WalletError::Operation)
    }

    pub async fn send_spl(
        &self,
        password: &str,
        mint: &str,
        to: &str,
        amount: f64,
    ) -> Result<SendResult, WalletError> {
        self.verify_password(password)?;
        let keypair = self.signing_keypair()?;
        self.rpc_handle()
            .send_spl(&keypair, mint, to, amount)
            .await
            .map_err(WalletError::Operation)
    }
}
