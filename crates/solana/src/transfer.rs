use anyhow::{Context, Result};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address,
    instruction::create_associated_token_account_idempotent,
};
use spl_token_interface::instruction as token_instruction;

pub(crate) fn build_sol_transfer(
    from: &Keypair,
    to: &Pubkey,
    lamports: u64,
    blockhash: solana_sdk::hash::Hash,
) -> Result<Transaction> {
    let instruction = system_instruction::transfer(&from.pubkey(), to, lamports);
    let mut tx = Transaction::new_with_payer(&[instruction], Some(&from.pubkey()));
    tx.message.recent_blockhash = blockhash;
    tx.sign(&[from], blockhash);
    Ok(tx)
}

pub(crate) fn build_spl_transfer(
    from: &Keypair,
    mint: &Pubkey,
    to: &Pubkey,
    amount: u64,
    blockhash: solana_sdk::hash::Hash,
) -> Result<Transaction> {
    let source_ata = get_associated_token_address(&from.pubkey(), mint);
    let destination_ata = get_associated_token_address(to, mint);

    let create_ata_ix =
        create_associated_token_account_idempotent(&from.pubkey(), to, mint, &spl_token_interface::id());
    let transfer_ix = token_instruction::transfer(
        &spl_token_interface::id(),
        &source_ata,
        &destination_ata,
        &from.pubkey(),
        &[],
        amount,
    )
    .context("failed to build SPL transfer instruction")?;

    let mut tx = Transaction::new_with_payer(
        &[create_ata_ix, transfer_ix],
        Some(&from.pubkey()),
    );
    tx.message.recent_blockhash = blockhash;
    tx.sign(&[from], blockhash);
    Ok(tx)
}
