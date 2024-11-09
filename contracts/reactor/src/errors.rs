use soroban_sdk::contracterror;

// TODO clean up errors

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    AlreadyDiscovered = 1,
    MineNotFound = 2,
    PailAmountTooLow = 3,
    AlreadyHasPail = 4,
    MineIsPaused = 5,
    HashIsInvalid = 6,
    BlockNotFound = 7,
    TooSoonToClaim = 8,
    KaleNotFound = 9,
    PailNotFound = 10,
    ZeroCountTooLow = 11,
}
