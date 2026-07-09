use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{SendPreview, SendResult};
use tauri::State;

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
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
