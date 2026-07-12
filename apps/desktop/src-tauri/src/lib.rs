mod commands;
mod error;
mod state;

use std::path::{Path, PathBuf};

use state::AppState;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

/// Previous product id — used only to migrate wallets/config after the rename.
const LEGACY_APP_IDENTIFIER: &str = "com.aegis.wallet";

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::generate_mnemonic,
        commands::validate_mnemonic,
        commands::create_wallet,
        commands::import_wallet,
        commands::unlock_wallet,
        commands::lock_wallet,
        commands::reveal_mnemonic,
        commands::remove_wallet,
        commands::change_wallet_password,
        commands::export_wallet,
        commands::export_wallet_to_path,
        commands::change_wallet_network,
        commands::get_wallet_snapshot,
        commands::get_sol_balance,
        commands::get_token_balances,
        commands::get_activity,
        commands::preview_sol_send,
        commands::preview_spl_send,
        commands::send_sol,
        commands::send_spl,
        commands::resolve_token,
        commands::search_tokens,
        commands::preview_swap_quote,
        commands::execute_swap,
        commands::get_app_settings,
        commands::update_app_settings,
        commands::get_managed_default_rpc_url,
        commands::set_onboarding_draft,
        commands::get_onboarding_draft,
        commands::clear_onboarding_draft,
    ])
}

fn typescript_exporter() -> specta_typescript::Typescript {
    specta_typescript::Typescript::default()
        .header("// @ts-nocheck")
        .bigint(specta_typescript::BigIntExportBehavior::Number)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// One-time: if the new data dir is empty, copy wallets + config from the old Aegis path.
fn migrate_legacy_app_data(new_dir: &Path) {
    let wallets_new = new_dir.join("wallets");
    let config_new = new_dir.join("config.json");
    if wallets_new.exists() || config_new.exists() {
        return;
    }

    let Some(parent) = new_dir.parent() else {
        return;
    };
    let old_dir: PathBuf = parent.join(LEGACY_APP_IDENTIFIER);
    if !old_dir.is_dir() {
        return;
    }

    let wallets_old = old_dir.join("wallets");
    if wallets_old.is_dir() {
        let _ = copy_dir_recursive(&wallets_old, &wallets_new);
    }
    let config_old = old_dir.join("config.json");
    if config_old.is_file() {
        let _ = std::fs::copy(&config_old, &config_new);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(typescript_exporter(), "../src/bindings.ts")
        .expect("failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&data_dir).ok();
            migrate_legacy_app_data(&data_dir);
            // Managed product default comes from RuntimeConfig; env is dev-only overlay.
            let wallet = wallet_core::WalletService::new(&data_dir, None);
            app.manage(AppState::new(wallet));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_typescript_bindings() {
        specta_builder()
            .export(typescript_exporter(), "../src/bindings.ts")
            .expect("failed to export TypeScript bindings");
    }
}
