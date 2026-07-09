use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{ActivityItem, SendPreview, SendResult, TokenBalance, WalletFile};
use tauri::State;

#[tauri::command]
pub fn wallet_exists(state: State<'_, AppState>) -> bool {
    state.wallet.wallet_exists()
}

#[tauri::command]
pub fn generate_mnemonic(state: State<'_, AppState>) -> CommandResult<String> {
    state.wallet.generate_mnemonic().map_err(map_wallet_error)
}

#[tauri::command]
pub fn create_wallet(
    mnemonic: String,
    password: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletFile> {
    state
        .wallet
        .create_wallet(&mnemonic, &password)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn import_wallet(
    mnemonic: String,
    password: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletFile> {
    state
        .wallet
        .import_wallet(&mnemonic, &password)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn unlock_wallet(password: String, state: State<'_, AppState>) -> CommandResult<String> {
    state.wallet.unlock(&password).map_err(map_wallet_error)
}

#[tauri::command]
pub fn lock_wallet(state: State<'_, AppState>) {
    state.wallet.lock();
}

#[tauri::command]
pub fn is_unlocked(state: State<'_, AppState>) -> bool {
    state.wallet.is_unlocked()
}

#[tauri::command]
pub fn get_public_key(state: State<'_, AppState>) -> Option<String> {
    state.wallet.get_public_key()
}

#[tauri::command]
pub fn reveal_mnemonic(password: String, state: State<'_, AppState>) -> CommandResult<String> {
    state
        .wallet
        .reveal_mnemonic(&password)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn get_wallet_snapshot(
    state: State<'_, AppState>,
) -> CommandResult<models::WalletSnapshot> {
    state.wallet.get_snapshot().await.map_err(map_wallet_error)
}

#[tauri::command]
pub async fn get_sol_balance(state: State<'_, AppState>) -> CommandResult<f64> {
    state
        .wallet
        .get_sol_balance()
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn get_token_balances(state: State<'_, AppState>) -> CommandResult<Vec<TokenBalance>> {
    state
        .wallet
        .get_token_balances()
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn get_activity(
    limit: usize,
    state: State<'_, AppState>,
) -> CommandResult<Vec<ActivityItem>> {
    state
        .wallet
        .get_activity(limit)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn preview_sol_send(
    to: String,
    amount_sol: f64,
    state: State<'_, AppState>,
) -> CommandResult<SendPreview> {
    state
        .wallet
        .preview_sol_send(&to, amount_sol)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn preview_spl_send(
    mint: String,
    to: String,
    amount: f64,
    state: State<'_, AppState>,
) -> CommandResult<SendPreview> {
    state
        .wallet
        .preview_spl_send(&mint, &to, amount)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn send_sol(
    password: String,
    to: String,
    amount_sol: f64,
    state: State<'_, AppState>,
) -> CommandResult<SendResult> {
    state
        .wallet
        .send_sol(&password, &to, amount_sol)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
pub async fn send_spl(
    password: String,
    mint: String,
    to: String,
    amount: f64,
    state: State<'_, AppState>,
) -> CommandResult<SendResult> {
    state
        .wallet
        .send_spl(&password, &mint, &to, amount)
        .await
        .map_err(map_wallet_error)
}
