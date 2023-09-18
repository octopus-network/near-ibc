# near-ibc

This is an implementation of the IBC protocol (IBC/TAO) in the NEAR protocol, which includes additional applications such as ICS-20, written by Octopus Network.

## Implementation of IBC/TAO

The `near-ibc` crate is a NEAR smart contract that contains the implementation of interfaces (traits) defined in [ibc-rs](https://github.com/cosmos/ibc-rs). These interfaces are essential for IBC/TAO processes. The smart contract also offers view functions for IBC relayer [hermes](https://github.com/informalsystems/hermes). These functions enable querying of the state of hosted clients, connections, channels and other necessary IBC data.

## Implementation of ICS-20

The `near-ibc` crate also includes the implementation of the `transfer` module (ICS-20) to reduce the impact of current `ibc-rs` implementation.

Our implementation of the `BankKeeper` trait uses sub-accounts mechanism of NEAR protocol. The general design is as the following:

![NEAR IBC accounts](/images/near-ibc-accounts.png)

### Root account

The root account will be deployed by the wasm of the `near-ibc` crate. It includes the whole implementation of IBC/TAO and application module `transfer` (ICS-20).

The contract `near-ibc` will at least provide the following interfaces (functions):

* Function `deliver`:
  * Any account can call this function.
  * This function is for relayers to deliver IBC packet to IBC/TAO implementation. It will perform full standard processes for IBC packet implemented by `ibc-rs` crate.
* Function `setup_wrapped_token`:
  * Only the governance account can call this function.
  * This function will call `setup_asset` function of `token-factory` contract to create and initialize a wrapped token contract for a specific asset from a certain channel.
* Function `setup_channel_escrow`:
  * Only the governance account can call this function.
  * This function will call `create_escrow` function of `escrow-factory` contract to create and initialize an escrow contract for a specific channel.
* Function `register_asset_for_channel`:
  * Only the governance account can call this function.
  * This function will call `register_asset` function of `channel-escrow` contract to register a `token contract` and its `denom` as a whitelisted asset for a certain channel.
* Function `process_transfer_request`:
  * Only the sub-accounts of `near-ibc` account can call this function.
  * This function will call the `send_transfer` function implemented in `ibc-rs` crate to update on-chain state and generate necessary IBC events for relayers to perform a cross-chain token transfer. (Refer to [Sub accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains) and [Sub accounts for channel escrows](#sub-accounts-for-channel-escrows) for more details.)

### Sub account `transfer`

Full account id: `transfer.<root account>`.

The sub-accounts at this level are reserved for applications (modules) in IBC protocol. In our implementation of ICS-20, this sub-account doesn't require any smart contract code deployment, as it is only acting as a placeholder.

### Sub account `token-factory`

Full account id: `tf.transfer.<root account>`.

This account is for deploying token contracts for assets from other chains. The contract in this account is implemented by the `token-factory` crate.

The contract `token-factory` will at least provide the following interfaces (functions):

* Function `setup_asset`:
  * Only the ancestor accounts of current account can call this function. The original caller should be the governance account set in `near-ibc` contract.
  * This function checks whether the sub-account for the asset corresponding to the `denomination` of the coin (passed by the caller) exists. If not, a new sub-account will be created and initialized automatically.
  * When it is necessary to create sub-account for a new asset, this function will also check for duplication of both `denomination` and `asset id` (refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains)) in order to avoid hash collisions. Besides, the mappings of `denomination` and `asset id` will also be stored in this contract.
* Function `mint_asset`:
  * Only the ancestor accounts of current account can call this function.
  * This function will be called in function `BankKeeper::mint_coins`, which is implemented by the `transfer` module in `near-ibc` contract.
  * This function will call the `mint` function of the contract of the sub-account automatically. (Also refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains).)
* Necessary view functions for querying `denomination`s and `asset id`s.

### Sub accounts for assets from other chains

Full account id: `<asset id>.tf.transfer.<root account>`, where the `asset id` is the leftmost 128 bits of sha256 hash of the denomination of a certain cross-chain asset, in hex format. Thus, the length of the whole account id will be `32 + 13 + (length of root account id)`, which can be controlled not longer than `64`.

This account is for minting and burning cross-chain assets that are NOT native in NEAR protocol. The contract in this account is implemented by the `wrapped-token` crate.

The contract `wrapped-token` will at least provide the following interfaces (functions):

* Function `mint`:
  * Only the parent account of current account (the `token-factory` account) can call this function.
  * This function will mint a given amount of tokens to a given account in current token contract.
* Function `request_transfer`:
  * Only the token holders of in this contract can call this function.
  * If all checks passed, this function will lock the given amount of tokens from the caller account (internal transfer them to the current account) and generate a `pending transfer request` for the caller account. Then it will schedule a call of `process_transfer_request` function of `near-ibc` contract.
* Function `apply_transfer_request`:
  * Only the `near-ibc` contract account can call this function.
  * If the given parameters matches the `pending transfer request` of the given user account, the `pending transfer request` will be applied and removed. The given amount of tokens will be internal burnt from the current account.
* Function `cancel_transfer_request`:
  * Only the `near-ibc` contract account can call this function.
  * If the given parameters matches the `pending transfer request` of the given user account, the `pending transfer request` will be canceled and removed. The given amount of tokens will be unlocked (internal transferred from the current account to the caller account corresponding to the `pending transfer request`).

### Sub account `escrow-factory`

Full account id: `ef.transfer.<root account>`.

This account is for deploying escrow contracts for assets native in NEAR protocol. The contract in this account is implemented by the `escrow-factory` crate.

The contract `escrow-factory` will at least provide the following interfaces (functions):

* Function `create_escrow`:
  * Only the ancestor accounts of current account can call this function.
  * This function will create a sub-account for a certain IBC channel if it does not already exist. Then deploy and initialize the escrow contract (implemented by crate `channel-escrow`) in the sub-account automatically.

### Sub accounts for channel escrows

Full account id: `<channel id>.ef.transfer.<root account>`.

This account is for receiving/locking NEP-141 assets that are native in NEAR protocol when they are transferred out of the NEAR protocol. It is also responsible for transferring/unlocking these NEP-141 assets when they are transferred back to the NEAR protocol. The contract in this account is implemented by the `channel-escrow` crate.

The contract `channel-escrow` will at least provide the following interfaces (functions):

* Function `register_asset`:
  * Only the `near-ibc` contract account can call this function. The original caller should be the governance account set in `near-ibc` contract.
  * This function stores the given `token contract` and its `denom` as a whitelisted asset.
* Function `ft_on_transfer`:
  * This function is for receiving assets (whose source chain is the NEAR protocol) from the NEAR protocol. It acts as a callback function which will be triggered when a token transfer to this account happens by calling the `ft_transfer_call` function of any NEP-141 contract.
  * Only the transfers from `registered token contracts` will be accepted.
  * If all checks passed, this function will generate a `pending transfer request` for the sender account. Then it will schedule a call of `process_transfer_request` function of `near-ibc` contract.
* Function `apply_transfer_request`:
  * Only the `near-ibc` contract account can call this function.
  * If the given parameters matches the `pending transfer request` of the given user account, the `pending transfer request` will be applied and removed.
* Function `cancel_transfer_request`:
  * Only the `near-ibc` contract account can call this function.
  * If the given parameters matches the `pending transfer request` of the given user account, the `pending transfer request` will be canceled and removed. The given amount of tokens will be transferred back to the sender account corresponding to the `pending transfer request`.
* Function `do_transfer`:
  * Only the `near-ibc` contract account can call this function.
  * The `BankKeeper::send_coins` function, implemented by the `transfer` module in `near-ibc` contract, will call this function to transfer a certain amount of previously locked NEP-141 tokens from current account to a specific receiver in the NEAR protocol.

### General process of ICS20 implementation

#### Transfer asset from other chain to NEAR protocol

![3-1](/images/near_ibc-Page-3-1.drawio.png)

#### Redeem cross-chain asset from NEAR protocol back to the source chain

![3-2](/images/near_ibc-Page-3-2.drawio.png)

#### Transfer asset from NEAR protocol to other chains

![4-1](/images/near_ibc-Page-4-1.drawio.png)

#### Redeem cross-chain asset from other chains back to NEAR protocol

![4-2](/images/near_ibc-Page-4-2.drawio.png)

## Supporting features

Please refer to release notes for details.

* [v1.0.0 pre-release 1](https://github.com/octopus-network/near-ibc/releases/tag/v1.0.0-pre.1)

## Auditing

These contracts had completed auditing by:

* [Blocksec](https://blocksec.com) - The report is [here](/auditing/blocksec_near-ibc_v1.0_signed.pdf).
