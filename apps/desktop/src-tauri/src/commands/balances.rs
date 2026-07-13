use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{ActivityItem, TokenBalance, WalletSnapshot};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn get_wallet_snapshot(state: State<'_, AppState>) -> CommandResult<WalletSnapshot> {
    state.wallet.get_snapshot().await.map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn get_sol_balance(state: State<'_, AppState>) -> CommandResult<f64> {
    state
        .wallet
        .get_sol_balance()
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn get_token_balances(state: State<'_, AppState>) -> CommandResult<Vec<TokenBalance>> {
    state
        .wallet
        .get_token_balances()
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
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
