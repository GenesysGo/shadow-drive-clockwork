pub const SDRIVE_OBJECT_PREFIX: &'static str =
    "https://shdw-drive.genesysgo.net";

pub const TIME_DELAY_SECS: i64 = 0;

pub mod shdw {
    #[cfg(feature = "mainnet")]
    anchor_lang::declare_id!("SHDWyBxihqiCj6YekG2GUr7wqKLeLAMK1gHZck9pL6y");
    #[cfg(not(feature = "mainnet"))]
    anchor_lang::declare_id!("RUNEkHeD5P8DsSpuDwxyZZKsj3T9e1ooMiaXL9H71yc");
}
