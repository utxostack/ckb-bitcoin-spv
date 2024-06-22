# CKB Bitcoin SPV Design

CKB Bitcoin SPV is a library designed to facilitate Bitcoin simplified payment verification (SPV) on Nervos CKB.

## Abstract

This document provides a comprehensive overview of the design and technical details of the CKB Bitcoin SPV. Through this material, developers will acquire the knowledge needed to perform on-chain verification of Bitcoin transactions on Nervos CKB.

## Background

A basic knowledge of the following concepts is **required** for a better understanding of this document. 

Below are brief explanations of each term along with links to sources for detailed learning:

### Simplified Payment Verification (SPV)

[Simple Payment Verification][SPV] (SPV) allows a transaction receiver to confirm that the sender has control of the source funds of the payment they are offering, without needing to download the entire blockchain. This is achieved using Merkle proofs.

Refer to the [Bitcoin whitepaper](https://bitcoin.org/bitcoin.pdf) for more details.

### Bitcoin Difficulty Adjustment

Bitcoin adjusts the computational difficulty of mining a block every 2016 blocks, which ideally takes two weeks. Every Bitcoin client
adjusts the mining difficulty by comparing the actual production time of the last 2016 blocks to the intended 2-week period, and modifies the target based on the percentage difference. [^1]

> [!Note]
> “The difficulty re-target [is] based on the time taken for the previous 2015 blocks instead of 2016 blocks." [^2]

### Bitcoin Merkle Proofs

A Bitcoin Merkle proof verifies that a transaction - or specifically, its hash - was included in a given block. It can be fetched
through the RPC API [`gettxoutproof`] and verified with RPC API [`verifytxoutproof`].

### Merkle Mountain Range (MMR)

A [Merkle Mountain Range (MMR)][MMR] is a binary hash tree data structure designed to allow efficient appending of new leaves while
maintaining the integrity of the existing nodes. 

A MMR proof can be utilized to verify whether a specific item is included in the MMR root.

## Overview

Let’s break down the entire problem into two smaller, independent steps.

### 1. On CKB, prove if a header belongs to the bitcoin chain

#### 1.1. Data preparation stage

Since we want to do on-chain verification, so the resources are limited, for
example, we couldn't afford 100 MiB storage or 30 seconds expensive
computation, on CKB.

Given on-chain verification comes with resource constraints, such as the inability to afford 100 MiB of storage or 30 seconds of
computation on CKB, the MMR is introduced to address this issue by only saving the MMR root of Bitcoin headers on CKB.

Here's how it works:

1. initialize a cell with a Bitcoin header at any height.

   An MMR tree is constructed with the header, and its root is saved in the cell data.
   No on-chain verification is performed during this initialization; users need to verify the data off-chain and then trust it.

3. [ckb-bitcoin-spv-service](https://github.com/ckb-cell/ckb-bitcoin-spv-service) will build the same MMR tree off-chain.

   This service will listen to the Bitcoin blockchain for new blocks. When a new Bitcoin block is mined, the service will update the
   MMR tree with the new block, calculate a new MMR root, and then send both the new MMR root and the new block header to the CKB chain.

4. an on-chain [script](https://github.com/ckb-cell/ckb-bitcoin-spv-contracts/tree/master/contracts/ckb-bitcoin-spv-type-lock) performs the following checks:

   1) Check the new header with two parts:

    - The field "previous block header hash" in headers[^3] should be correct;

    - The POW for the block should be valid.

      For security, the on-chain script calculates the POW target for
      the next block, requiring the on-chain caching of:

      - The start time of the first block after difficulty adjustment.

      - If the next block is one of the first blocks after difficulty
        adjustment, its target should be calculated and cached.

    2) Check the new MMR root:

  - The new MMR root should be based on the previous MMR root with only the new header appended.

  Once these checks are passed, the new data is saved into the cell.

> [!NOTE]
> Bitcoin headers do not store the height,
> but all heights must be stored on CKB chain for two reasons:
> calculating the MMR index and determining block confirmations.
> calculate the block confirmations.

#### 1.2. Verification stage

With the stored MMR root on CKB, an on-chain script can verify whether a
Bitcoin header is part of the MMR tree.

The following data are required to be submitted to the CKB chain:
- The MMR proof of the header to be proven.
- The full data of the header, or its hash.
- The height of the header.

If the PoW of the header is verified and the header is within the MMR tree on CKB, it confirms that the corresponding header is part of
the Bitcoin chain.

### 2. On CKB, prove if a transaction is in a Bitcoin block

#### 2.1. Data preparation stage

No additional data is required to be stored on CKB chain for transactions
verification.

#### 2.2. Verification stage

A Bitcoin header[^3] contains a field called "merkle root hash".

A merkle root is derived from the hashes of all transactions in
that block, ensuring that none of those transactions can be modified without
modifying the header.

Thus, a transaction can be verified whether it's in a header, with the merkle root hash and a merkle proof.

## Optimization

Storing data on-chain will permanently occupy the capacity of CKBytes. Since not all Bitcoin headers will be used, not all Bitcoin
headers will be saved on the chain. For verification, they can be included in the `witnesses`.

When verifying a Bitcoin header, that header or only its hash should be submitted to the CKB chain. 

However, when verifying a Bitcoin transaction, the full header must be submitted to the CKB chain because the "merkle root hash" in the
header is required. An interesting fact is that the merkle proof of the transaction already contains the full header [^4] , so the
header doesn't have to be submitted separately.

## Disadvantages

- Calculate the MMR proof is complex for average users.

  A service is needed to collect all headers contained in the MMR root.

## References

- [`CalculateNextWorkRequired(..)`]: This function is used to calculate the next target.

- [`CPartialMerkleTree::ExtractMatches(..)`]: This function ensures that the partial Merkle tree is correctly constructed.
  It is used to verify that a proof points to transactions.

- [Merkle Mountain Ranges][MMR]

<!--

    Links

  -->

[^1]: [Bitcoin Target](https://en.bitcoin.it/wiki/Target)
[^2]: [Gaming the "off-by-one" bug (difficulty re-target based on 2015 instead of 2016 block time span)?](https://bitcoin.stackexchange.com/questions/1511)
[^3]: [Bitcoin Block Headers](https://developer.bitcoin.org/reference/block_chain.html#block-headers)
[^4]: [Bitcoin serialization format: Merkle proof](https://daniel.perez.sh/blog/2020/bitcoin-format/#merkle-proof)

[`gettxoutproof`]: https://developer.bitcoin.org/reference/rpc/gettxoutproof.html
[`verifytxoutproof`]: https://developer.bitcoin.org/reference/rpc/verifytxoutproof.html

[`CalculateNextWorkRequired(..)`]: https://github.com/bitcoin/bitcoin/blob/v26.0/src/pow.cpp#L49
[`CPartialMerkleTree::ExtractMatches(..)`]: https://github.com/bitcoin/bitcoin/blob/v26.0/src/merkleblock.cpp#L149

[SPV]: https://bitcoinwiki.org/wiki/simplified-payment-verification
[MMR]: https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md
