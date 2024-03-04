#[repr(i8)]
pub enum BootstrapError {
    // Basic errors.
    DecodeHeader = 1,
    // Check data.
    Height = 9,
    Pow,
    // This is not an error, just make sure the error code is less than 32.
    Unreachable = 32,
}

#[repr(i8)]
pub enum UpdateError {
    // Basic errors.
    DecodeHeader = 1,
    DecodeTargetAdjustInfo,
    // Check headers.
    EmptyHeaders = 9,
    UncontinuousHeaders,
    Difficulty,
    Pow,
    // Check MMR proof.
    Mmr = 17,
    HeadersMmrProof,
    // Check new client.
    ClientId = 25,
    ClientTipBlockHash,
    ClientMinimalHeight,
    ClientMaximalHeight,
    ClientTargetAdjustInfo,
    // This is not an error, just make sure the error code is less than 32.
    Unreachable = 32,
}

#[repr(i8)]
pub enum VerifyTxError {
    // Basic errors.
    DecodeTransaction = 1,
    DecodeTxOutProof,
    // Check
    TxOutProof = 9,
    HeaderMmrProof,
    // This is not an error, just make sure the error code is less than 32.
    Unreachable = 32,
}
