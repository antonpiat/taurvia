use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{Network, RuntimeConfig, WalletFile};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn generate_mnemonic(state: State<'_, AppState>) -> CommandResult<String> {
    state.wallet.generate_mnemonic().map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn validate_mnemonic(mnemonic: String, state: State<'_, AppState>) -> CommandResult<()> {
    state
        .wallet
        .validate_mnemonic(&mnemonic)
        .map_err(map_wallet_error)
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
pub fn reveal_mnemonic(password: String, state: State<'_, AppState>) -> CommandResult<String> {
    state
        .wallet
        .reveal_mnemonic(&password)
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn remove_wallet(password: String, state: State<'_, AppState>) -> CommandResult<()> {
    state
        .wallet
        .remove_wallet(&password)
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn change_wallet_password(
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    state
        .wallet
        .change_password(&old_password, &new_password)
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn export_wallet(password: String, state: State<'_, AppState>) -> CommandResult<String> {
    state
        .wallet
        .export_wallet(&password)
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn export_wallet_to_path(
    password: String,
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let json = state
        .wallet
        .export_wallet(&password)
        .map_err(map_wallet_error)?;
    std::fs::write(&path, json).map_err(|e| models::ApiError::new(
        "io_error",
        format!("failed to write wallet backup: {e}"),
    ))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_wallet_network(
    network: Network,
    state: State<'_, AppState>,
) -> CommandResult<RuntimeConfig> {
    state
        .wallet
        .change_network(network)
        .map_err(map_wallet_error)
}
