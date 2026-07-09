use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::WalletFile;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn wallet_exists(state: State<'_, AppState>) -> bool {
    state.wallet.wallet_exists()
}

#[tauri::command]
#[specta::specta]
pub fn generate_mnemonic(state: State<'_, AppState>) -> CommandResult<String> {
    state.wallet.generate_mnemonic().map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
pub fn unlock_wallet(password: String, state: State<'_, AppState>) -> CommandResult<String> {
    state.wallet.unlock(&password).map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn lock_wallet(state: State<'_, AppState>) {
    state.wallet.lock();
}

#[tauri::command]
#[specta::specta]
pub fn is_unlocked(state: State<'_, AppState>) -> bool {
    state.wallet.is_unlocked()
}

#[tauri::command]
#[specta::specta]
pub fn get_public_key(state: State<'_, AppState>) -> Option<String> {
    state.wallet.get_public_key()
}

#[tauri::command]
#[specta::specta]
pub fn reveal_mnemonic(password: String, state: State<'_, AppState>) -> CommandResult<String> {
    state
        .wallet
        .reveal_mnemonic(&password)
        .map_err(map_wallet_error)
}
