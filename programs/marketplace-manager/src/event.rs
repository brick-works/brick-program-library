use anchor_lang::prelude::*;

#[event]
pub struct BonusEvent {
    pub receiver: String,
    pub mint: String,
    pub amount: u64
}
// to-do