use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{SwapQuote, SwapResult, TokenInfo};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn resolve_token(mint: String, state: State<'_, AppState>) -> CommandResult<TokenInfo> {
    state
        .wallet
        .resolve_token(&mint)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn search_tokens(
    query: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TokenInfo>> {
    state
        .wallet
        .search_tokens(&query)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn preview_swap_quote(
    input_mint: String,
    output_mint: String,
    amount_ui: f64,
    slippage_bps: u16,
    state: State<'_, AppState>,
) -> CommandResult<SwapQuote> {
    state
        .wallet
        .preview_swap(&input_mint, &output_mint, amount_ui, slippage_bps)
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn execute_swap(
    password: String,
    input_mint: String,
    output_mint: String,
    amount_ui: f64,
    slippage_bps: u16,
    state: State<'_, AppState>,
) -> CommandResult<SwapResult> {
    state
        .wallet
        .execute_swap(
            &password,
            &input_mint,
            &output_mint,
            amount_ui,
            slippage_bps,
        )
        .await
        .map_err(map_wallet_error)
}
