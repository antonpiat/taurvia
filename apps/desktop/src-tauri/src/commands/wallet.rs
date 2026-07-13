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
pub fn import_wallet_backup(
    wallet_json: String,
    password: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletFile> {
    state
        .wallet
        .import_wallet_backup(&wallet_json, &password)
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
pub fn device_protection_enabled(state: State<'_, AppState>) -> bool {
    state.wallet.device_protection_enabled()
}

#[tauri::command]
#[specta::specta]
pub fn enable_device_protection(
    password: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletFile> {
    state
        .wallet
        .enable_device_protection(&password)
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn disable_device_protection(
    password: String,
    state: State<'_, AppState>,
) -> CommandResult<WalletFile> {
    state
        .wallet
        .disable_device_protection(&password)
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
    let path = std::path::PathBuf::from(path.trim());
    if path.as_os_str().is_empty() {
        return Err(models::ApiError::new(
            "io_error",
            "backup path is required",
        ));
    }
    if !path.is_absolute() {
        return Err(models::ApiError::new(
            "io_error",
            "backup path must be absolute",
        ));
    }
    let Some(parent) = path.parent() else {
        return Err(models::ApiError::new(
            "io_error",
            "backup path has no parent directory",
        ));
    };
    if !parent.exists() {
        return Err(models::ApiError::new(
            "io_error",
            "backup directory does not exist",
        ));
    }

    let json = state
        .wallet
        .export_wallet(&password)
        .map_err(map_wallet_error)?;
    std::fs::write(&path, json).map_err(|e| {
        models::ApiError::new("io_error", format!("failed to write wallet backup: {e}"))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
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
