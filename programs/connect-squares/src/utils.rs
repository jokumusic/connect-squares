use anchor_lang::prelude::*;
use crate::errors::GameError;

pub fn transfer_owned_sol(from: &mut AccountInfo, to: &mut AccountInfo, amount: u64) -> Result<()> {
    let post_from = from
        .lamports()
        .checked_sub(amount)
        .ok_or(GameError::PayoutDebitNumericalOverflow)?;

    let post_to = to
        .lamports()
        .checked_add(amount)
        .ok_or(GameError::PayoutCreditNumericalOverflow)?;

    **from.try_borrow_mut_lamports().unwrap() = post_from;
    **to.try_borrow_mut_lamports().unwrap() = post_to;

    Ok(())
}
/*
pub fn transfer_sol<'a>(from: &'a AccountInfo, to: &'a AccountInfo, amount: u64) -> Result<()> {
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        from.key,
        to.key,
        amount,
    );

    let result = anchor_lang::solana_program::program::invoke(
        &ix,
        &[from.clone(), to.clone()]
    );

    match result {
        Err(_err)=> Err(_err.into()),
        Ok(()) => Ok(())
    }
}
*/
