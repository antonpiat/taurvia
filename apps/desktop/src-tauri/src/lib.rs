mod commands;
mod error;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&data_dir).ok();
            let rpc_url = std::env::var("AEGIS_RPC_URL").ok();
            let wallet = wallet_core::WalletService::new(&data_dir, rpc_url.as_deref());
            app.manage(AppState::new(wallet));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::wallet_exists,
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::import_wallet,
            commands::unlock_wallet,
            commands::lock_wallet,
            commands::is_unlocked,
            commands::get_public_key,
            commands::reveal_mnemonic,
            commands::get_wallet_snapshot,
            commands::get_sol_balance,
            commands::get_token_balances,
            commands::get_activity,
            commands::preview_sol_send,
            commands::preview_spl_send,
            commands::send_sol,
            commands::send_spl,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
