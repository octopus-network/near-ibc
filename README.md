# near-ibc

An implementation of IBC protocol (IBC/TAO and some applications, like ICS-20 etc.) in NEAR protocol, written by Octopus Network.

## Implementation of IBC/TAO

Crate `near-ibc` is a NEAR smart contract. It includes the implementation of interfaces (traits) defined in [ibc-rs](https://github.com/cosmos/ibc-rs) which are needed for basic IBC/TAO processes. This contract also provides view functions for IBC relayer [hermes](https://github.com/informalsystems/hermes) to query state of hosted clients, connections, channels and other necessary IBC data.

## Implementation of ICS-20

The implementation of module `transfer` (ICS-20) is also included in crate `near-ibc` to minimize the impact to current implementation of `ibc-rs`.

Our implementation of trait `BankKeeper` uses sub-accounts mechanism of NEAR protocol. The general design is as the following:

![NEAR IBC accounts](/images/near-ibc-accounts.png)

### Root account

The root account will be deployed by the wasm of crate `near-ibc`. It includes the whole implementation of IBC/TAO and application module `transfer` (ICS-20).

### Sub account `transfer`

Full account id: `transfer.<root account>`.

The sub-accounts at this level are reserved for applications (modules) in IBC protocol. In our implementation of ICS-20, this sub-account doesn't need to be deployed by any smart contract code. It's only acting as placeholder.

### Sub account `token-factory`

Full account id: `tf.transfer.<root account>`.

This account is for deploying token contracts for assets from other chains. The contract in this account is implemented by crate `token-factory`.

The contract `token-factory` will at least provide the following interfaces (functions):

* Function `mint_asset`:
  * This function can ONLY be called from root account.
  * This function will be called in function `BankKeeper::mint_coins` which is implemented by module `transfer`.
  * This function will check whether the sub-account for the asset corresponding to the `denomination` of the coin (passed by the caller) is existed. If not, a new sub-account will be created and initialized automatically. Then the function `mint` of the contract of the sub-account will be called automatically. (Also refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains).)
  * When it is necessary to create sub-account for a new asset, this function will also check the duplication for both `denomination` and `asset id` (refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains)) to avoid hash collisions. Besides, the mappings of `denomination` and `asset id` are also be stored in this contract.
* Function `burn_asset`:
  * This function can ONLY be called from root account.
  * This function will be called in function `BankKeeper::burn_coins` which is implemented by module `transfer`.
  * This function will call the function `burn` of the contract of the sub-account corresponding to the `denomination` of the coin (passed by the caller) automatically. (Also refer to [sub-accounts for assets from other chains](#sub-accounts-for-assets-from-other-chains).)
* Necessary view functions for querying `denomination`s and `asset id`s.

### Sub accounts for assets from other chains

Full account id: `<asset id>.tf.transfer.<root account>`, where the `asset id` is the leftmost 128 bits of sha256 hash of the denomination of a certain cross-chain asset, in hex format. So the length of the whole account id will be `32 + 13 + (length of root account id)`, which can be controlled not longer than `64`.

This account is for minting and burning cross-chain assets which are NOT native in NEAR protocol. The contract in this account is implemented by crate `wrapped-token`.

The contract `wrapped-token` will at least provide the following interfaces (functions):

* Function `mint`:
  * This function can ONLY be called from sub-account `token-factory` (the previous level of current account id).
  * This function will generate a certain IBC event to inform relayer that a certain amount of coins of a cross-chain asset are minted.
* Function `burn`:
  * This function can ONLY be called from sub-account `token-factory` (the previous level of current account id).
  * This function will generate a certain IBC event to inform relayer that a certain amount of coins of a cross-chain asset are burned.

### Sub account `escrow-factory`

Full account id: `ef.transfer.<root account>`.

This account is for deploying escrow contracts for assets native in NEAR protocol. The contract in this account is implemented by crate `escrow-factory`.

The contract `escrow-factory` will at least provide the following interfaces (functions):

* Function `create_escrow`:
  * This function can ONLY be called from root account.
  * This function will be called in callback function `Module::on_chan_open_confirm`.
  * This function will create a sub-account for a certain IBC channel if it is not exised. Then deploy and initialize the escrow contract (implemented by crate `channel-escrow`) in the sub-account automatically.

### Sub accounts for channel escrows

Full account id: `<channel id>.ef.transfer.<root account>`.

This account is for receiving/locking NEP-141 assets which are native in NEAR protocol when they are transferred out of NEAR protocol, and for transferring/unlocking these NEP-141 assets when they are transferred back to NEAR protocol. The contract in this account is implemented by crate `channel-escrow`.

The contract `channel-escrow` will at least provide the following interfaces (functions):

* Function `ft_on_transfer`:
  * This function is for receiving assets (which's source chain is NEAR protocol) from NEAR protocol. It is acting as a callback of the calling of function `ft_transfer_call` of any NEP-141 contract.
  * This function will generate a certain IBC event or call a certain function of contract `near-ibc` in root account to start the process for transferring NEAR native assets to other chains. **(TBD)**
* Function `transfer`:
  * This function can ONLY be called from root account.
  * This function will transfer a certain amount of previously locked NEP-141 tokens to a certain receiver in NEAR protocol.
  * This function will generate a certain IBC event to inform relayer that a certain amount of previously locked NEP-141 tokens are transferred.
