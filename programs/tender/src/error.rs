use anchor_lang::error_code;

#[error_code]
pub enum ErrorCode {
    #[msg("Wrong authority")]
    IncorrectAuthority,
    #[msg("Wrong owner on a token account")]
    IncorrectOwner,
    #[msg("Wrong mint on a token account")]
    IncorrectMint,
    #[msg("Error create account")]
    CreateAccountError,
    #[msg("Error not transferable mint cpi")]
    ErrorInitNotTransferable,
    #[msg("Extension not supported")]
    ExtensionNotSupported,
    #[msg("Error init mint cpi")]
    InitMintError,
    #[msg("Incorrect seeds")]
    IncorrectSeeds
}