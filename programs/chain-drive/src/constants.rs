pub const SDRIVE_OBJECT_PREFIX: &'static str =
    "https://shdw-drive.genesysgo.net";

pub const TIME_DELAY_SECS: i64 = 0;

pub mod shdw {
    #[cfg(feature = "mainnet")]
    anchor_lang::declare_id!("SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y");
    #[cfg(not(feature = "mainnet"))]
    anchor_lang::declare_id!("RUNEkHeD5P8DsSpuDwxyZZKsj3T9e1ooMiaXL9H71yc");
}

pub mod payout_authority {
    #[cfg(feature = "mainnet")]
    anchor_lang::declare_id!("D6wZ5U9onMC578mrKMp5PZtfyc5262426qKsYJW7nT3p");
    #[cfg(not(feature = "mainnet"))]
    anchor_lang::declare_id!("3cZiETXADiH4spd8rdBCt5DwuVQMPuNnq8e7Ci2ky65L");
}

pub fn payout_account() -> anchor_lang::prelude::Pubkey {
    anchor_spl::associated_token::get_associated_token_address(
        &payout_authority::ID,
        &shdw::ID,
    )
}
