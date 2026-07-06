use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{ActivityItem, SendPreview, SendResult, TokenBalance, WalletFile};
use tauri::State;

#[tauri::command]
pub fn wallet_exists(state: State<AppState>) -> bool {
    state.wallet.wallet_exists()
}

#[tauri::command]
pub fn generate_mnemonic(state: State<AppState>) -> CommandResult<String> {
    state.wallet.generate_mnemonic().map_err(map_wallet_error)
}

#[tauri::command]
pub fn create_wallet(
    mnemonic: String,
    password: String,
    state: State<AppState>,
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
    state: State<AppState>,
) -> CommandResult<WalletFile> {
    state
        .wallet
        .import_wallet(&mnemonic, &password)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn unlock_wallet(password: String, state: State<AppState>) -> CommandResult<String> {
    state.wallet.unlock(&password).map_err(map_wallet_error)
}

#[tauri::command]
pub fn lock_wallet(state: State<AppState>) {
    state.wallet.lock();
}

#[tauri::command]
pub fn is_unlocked(state: State<AppState>) -> bool {
    state.wallet.is_unlocked()
}

#[tauri::command]
pub fn get_public_key(state: State<AppState>) -> Option<String> {
    state.wallet.get_public_key()
}

#[tauri::command]
pub fn reveal_mnemonic(password: String, state: State<AppState>) -> CommandResult<String> {
    state
        .wallet
        .reveal_mnemonic(&password)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn get_sol_balance(state: State<AppState>) -> CommandResult<f64> {
    state.wallet.get_sol_balance().map_err(map_wallet_error)
}

#[tauri::command]
pub fn get_token_balances(state: State<AppState>) -> CommandResult<Vec<TokenBalance>> {
    state
        .wallet
        .get_token_balances()
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn get_activity(limit: usize, state: State<AppState>) -> CommandResult<Vec<ActivityItem>> {
    state
        .wallet
        .get_activity(limit)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn preview_sol_send(
    to: String,
    amount_sol: f64,
    state: State<AppState>,
) -> CommandResult<SendPreview> {
    state
        .wallet
        .preview_sol_send(&to, amount_sol)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn preview_spl_send(
    mint: String,
    to: String,
    amount: f64,
    state: State<AppState>,
) -> CommandResult<SendPreview> {
    state
        .wallet
        .preview_spl_send(&mint, &to, amount)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn send_sol(
    password: String,
    to: String,
    amount_sol: f64,
    state: State<AppState>,
) -> CommandResult<SendResult> {
    state
        .wallet
        .send_sol(&password, &to, amount_sol)
        .map_err(map_wallet_error)
}

#[tauri::command]
pub fn send_spl(
    password: String,
    mint: String,
    to: String,
    amount: f64,
    state: State<AppState>,
) -> CommandResult<SendResult> {
    state
        .wallet
        .send_spl(&password, &mint, &to, amount)
        .map_err(map_wallet_error)
}
