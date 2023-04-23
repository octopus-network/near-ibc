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

### Sub account `transfer`

Full account id: `transfer.<root account>`.

The sub-accounts at this level are reserved for applications (modules) in IBC protocol. In our implementation of ICS-20, this sub-account doesn't require any smart contract code deployment, as it is only acting as a placeholder.

### Sub account `token-factory`

Full account id: `tf.transfer.<root account>`.

This account is for deploying token contracts for assets from other chains. The contract in this account is implemented by the `token-factory` crate.

The contract `token-factory` will at least provide the following interfaces (functions):

* Function `mint_asset`:
  * Only the root account can call this function.
  * This function will be called in function `BankKeeper::mint_coins`, which is implemented by the `transfer` module.
  * This function checks whether the sub-account for the asset corresponding to the `denomination` of the coin (passed by the caller) exists. If not, a new sub-account will be created and initialized automatically. Then, the `mint` function of the contract of the sub-account will be called automatically. (Also refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains).)
  * When it is necessary to create sub-account for a new asset, this function will also check for duplication of both `denomination` and `asset id` (refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains)) in order to avoid hash collisions. Besides, the mappings of `denomination` and `asset id` will also be stored in this contract.
* Function `burn_asset`:
  * Only the root account can call this function.
  * This function will be called in function `BankKeeper::burn_coins`, which is implemented by the `transfer` module.
  * This function will call the function `burn` of the contract of the sub-account corresponding to the `denomination` of the coin (passed by the caller) automatically. (Also refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains).)
* Necessary view functions for querying `denomination`s and `asset id`s.

### Sub accounts for assets from other chains

Full account id: `<asset id>.tf.transfer.<root account>`, where the `asset id` is the leftmost 128 bits of sha256 hash of the denomination of a certain cross-chain asset, in hex format. Thus, the length of the whole account id will be `32 + 13 + (length of root account id)`, which can be controlled not longer than `64`.

This account is for minting and burning cross-chain assets that are NOT native in NEAR protocol. The contract in this account is implemented by the `wrapped-token` crate.

The contract `wrapped-token` will at least provide the following interfaces (functions):

* Function `mint`:
  * Only the sub-account `token-factory` (the previous level of current account id) can call this function.
  * This function will mint a specified amount of tokens owned by a specified account in current token contract.
  * This function will generate a certain IBC event to inform relayer that a specified amount of coins of a cross-chain asset have been minted.
* Function `burn`:
  * Only the sub-account `token-factory` (the previous level of current account id) can call this function.
  * This function will burn a specified amount of tokens owned by a specified account in current token contract.
  * This function will generate a certain IBC event to inform relayer that a specified amount of coins of a cross-chain asset have been burned.

### Sub account `escrow-factory`

Full account id: `ef.transfer.<root account>`.

This account is for deploying escrow contracts for assets native in NEAR protocol. The contract in this account is implemented by the `escrow-factory` crate.

The contract `escrow-factory` will at least provide the following interfaces (functions):

* Function `create_escrow`:
  * Only the root account can call this function.
  * This function is called in the `Module::on_chan_open_confirm` callback function when the channel creation process is complete.
  * This function will create a sub-account for a certain IBC channel if it does not already exist. Then deploy and initialize the escrow contract (implemented by crate `channel-escrow`) in the sub-account automatically.

### Sub accounts for channel escrows

Full account id: `<channel id>.ef.transfer.<root account>`.

This account is for receiving/locking NEP-141 assets that are native in NEAR protocol when they are transferred out of the NEAR protocol. It is also responsible for transferring/unlocking these NEP-141 assets when they are transferred back to the NEAR protocol. The contract in this account is implemented by the `channel-escrow` crate.

The contract `channel-escrow` will at least provide the following interfaces (functions):

* Function `ft_on_transfer`:
  * This function is for receiving assets (whose source chain is the NEAR protocol) from the NEAR protocol. It acts as a callback function that is called when the `ft_transfer_call` function of any NEP-141 contract is triggered.
  * This function will generate a certain IBC event or call a certain function of contract `near-ibc` in root account to start the process of transferring NEAR native assets to other chains. **(To be determined)**
* Function `transfer`:
  * Only the root account can call this function.
  * The `BankKeeper::send_coins` function, implemented by the `transfer` module, will call this function to transfer a specified amount of previously locked NEP-141 tokens to a specified receiver in the NEAR protocol.
  * This function will generate a certain IBC event to inform relayer that a specified amount of previously locked NEP-141 tokens are transferred.
