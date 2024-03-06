#[repr(i8)]
pub enum BootstrapError {
    // Basic errors.
    DecodeHeader = 0x01,
    // Check data.
    Height = 0x09,
    Pow,
    // This is not an error, just make sure the error code is less than 32.
    Unreachable = 0x20,
}

#[repr(i8)]
pub enum UpdateError {
    // Basic errors.
    DecodeHeader = 0x01,
    DecodeTargetAdjustInfo,
    // Check headers.
    EmptyHeaders = 0x09,
    UncontinuousHeaders,
    Difficulty,
    Pow,
    // Check MMR proof.
    Mmr = 0x11,
    HeadersMmrProof,
    // Check new client.
    ClientId = 0x19,
    ClientTipBlockHash,
    ClientMinimalHeight,
    ClientMaximalHeight,
    ClientTargetAdjustInfo,
    // This is not an error, just make sure the error code is less than 32.
    Unreachable = 0x20,
}

#[repr(i8)]
pub enum VerifyTxError {
    // Basic errors.
    DecodeTransaction = 0x01,
    DecodeTxOutProof,
    // Transaction related errors.
    TransactionUnconfirmed = 0x09,
    TransactionTooOld,
    TransactionTooNew,
    // Check txout proof.
    TxOutProofIsInvalid = 0x11,
    TxOutProofInvalidTxIndex,
    TxOutProofInvalidTxId,
    // Check header mmr proof.
    HeaderMmrProof = 0x19,
    // This is not an error, just make sure the error code is less than 32.
    Unreachable = 0x20,
}
