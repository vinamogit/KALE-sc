use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    HomesteadExists = 1,
    HomesteadMissing = 2,
    AssetAdminInvalid = 3,
    FarmPaused = 4,
    FarmNotPaused = 5,
    PlantAmountTooLow = 6,
    ZeroCountTooLow = 7,
    PailExists = 8,
    PailMissing = 9,
    WorkMissing = 10,
    BlockMissing = 11,
    HashInvalid = 12,
    HarvestNotReady = 13,
}
