# The Design of CKB Bitcoin SPV

CKB Bitcoin SPV is a library, for doing Bitcoin simplified payment
verification on CKB.

## Abstract

This document describes the design and explains the technical details of CKB
Bitcoin SPV, which allows users to do on-chain verification for bitcoin
transactions on CKB chain.

## Background

For understanding this document, the knowledge of the following concepts are
**required**.\
But we won't discuss them here, please learn them on their own documents.

### Simplified Payment Verification (SPV)

[Simple Payment Verification][SPV], usually abbreviated to SPV, is noted in
Bitcoin whitepaper. It allows a transaction recipient to prove that the
sender has control of the source funds of the payment they are offering
without downloading the full Blockchain, by utilizing the properties of
Merkle proofs.

### Bitcoin Difficulty Adjustments

Every 2016 blocks (which should take two weeks if this goal is kept
perfectly), every Bitcoin client compares the actual time it took to
generate these blocks with the 2 weeks goal and modifies the target by the
percentage difference.[^1]

> [!Note]
> The difficulty re-target being based on the time taken for the previous
> 2015 blocks instead of 2016 blocks.[^2]

### Bitcoin Merkle Proof

A proof that proves a transaction (in fact, just transaction hash) was
included in a block.

It could be fetched through the RPC API [`gettxoutproof`], and be verified
with the RPC API [`verifytxoutproof`].

### Merkle Mountain Range (MMR)

A [Merkle Mountain Range (MMR)][MMR] is a binary hash tree that allows for
efficient appends of new leaves without changing the value of existing nodes.

An MMR proof could be used to prove whether an item is included in an MMR
root or not.

## Overview

First, we could divide the whole problem into 2 smaller and standalone problems.

### 1. On CKB, prove a header belongs to the bitcoin chain

#### 1.1. Data preparation stage

Since we want to do on-chain verification, so the resources are limited, for
example, we couldn't afford 100 MiB storage or 30 seconds expensive
computation, on CKB.

So, we introduce the [Merkle Mountain Range (MMR)][MMR] to solve this
problem: we only save the MMR root of bitcoin headers on CKB.

- First, we initialize a cell with a bitcoin header at any height.

  An MMR tree will be constructed with this header only, and its root will
  be saved in the cell data.

  No on-chain verification will be done for this data.
  Users have to check the data off-chain, then trust it.

- Also, there will be a service, which builds the same MMR tree but off-chain.

  This service will listen to the bitcoin chain and wait for the next block.

  When the next bitcoin block is mined, this service will append this new
  bitcoin block into the MMR tree, calculate a new MMR root, then send the
  new MMR root and the new header to the CKB chain.

- An on-chain script will check those new data.

  - First, check the new header with two parts:

    - The field "previous block header hash" in headers[^3] should be correct.

    - The POW for the block should be valid.

      For security, the on-chain script has to calculate the POW target for
      the next block by itself.

      So, there are two more data have to be cached on chain:

      - The start time of the first block after difficulty adjustment.

      - If the next block is one of the first blocks after difficulty
        adjustment, its target should be calculated and cached.

  - Then, check the new MMR root:

    - The new MMR root should be constructed based on the previous MMR root,
      and only the new header should be appended.

  After the above checks passed, then save the new data into the cell.

> [!NOTE]
> In bitcoin headers, the height is not stored, but we have to store all
> heights on CKB chain.\
> The heights are required for two reasons: calculate the index of MMR and
> calculate the block confirmations.

#### 1.2. Verification stage

With the stored MMR root on CKB, an on-chain script can check whether a
bitcoin header is contained in that MMR tree.

There are 3 items required to be submitted to the CKB chain:
- The MMR proof of the header which is required to be proven.
- The full data of the header, or its hash.
- The height of the header.

As the PoW of the header has been verified, if a header is contained within the
MMR tree on CKB, it implies that the corresponding header is part of the Bitcoin
chain.

### 2. On CKB, prove a transaction is in a bitcoin block

#### 2.1. Data preparation stage

No more data is required to be stored in the CKB chain for transactions
verification.

#### 2.2. Verification stage

There is a field "merkle root hash" in bitcoin headers[^3].

A merkle root is derived from the hashes of all transactions included in
that block, ensuring that none of those transactions can be modified without
modifying the header.

So, a transaction could be verified whether it's in a header, with the
merkle root hash and a merkle proof which contains it.

## Optimization

Save any data on chain will occupy the capacity of CKBytes permanently. And,
not all bitcoin headers will be used.\
So, we won't save all bitcoin headers on the chain.\
For verification, we can just put them into the witnesses.

When users want to verify a bitcoin header, they should submit that header
to CKB chain, or only its hash is enough.

But when verifies a bitcoin transaction, the full header is required to be
submitted to CKB chain; because the "merkle root hash" in the header is
required. \
An interesting fact is, that the merkle proof of the transaction already
contains the full header.[^4] So the header doesn't have to be submitted
alone.

## Disadvantages

- Calculate the MMR proof is not simple for normal users.

  A service is required, to collect all headers which are contained in the
  MMR root.

## References

- [`CalculateNextWorkRequired(..)`]

  The function, which is used to calculate the next target.

- [`CPartialMerkleTree::ExtractMatches(..)`]

  The function ensures that the partial Merkle tree is correctly constructed.
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
