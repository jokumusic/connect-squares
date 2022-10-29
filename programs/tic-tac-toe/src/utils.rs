use anchor_lang::prelude::*;
use crate::errors::GameError;

pub fn transfer_sol(from: &mut AccountInfo, to: &mut AccountInfo, amount: u64) -> Result<()> {
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
