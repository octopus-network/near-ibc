#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export ACCOUNT_ID=v9.nearibc.testnet
#
#
#
# cp ~/.near-credentials/testnet/nearibc.testnet.json ~/.near-credentials/testnet/$ACCOUNT_ID.json
# sed -i '' "s/nearibc.testnet/$ACCOUNT_ID/" ~/.near-credentials/testnet/$ACCOUNT_ID.json
# near create-account $ACCOUNT_ID --masterAccount nearibc.testnet --initialBalance 50 --publicKey "ed25519:2o5tiq68jntS8hunVjhsMfEcXnStecYf6TamQSg28ffz"
#
#
#
# cp ~/.near-credentials/testnet/$ACCOUNT_ID.json ~/.near-credentials/testnet/transfer.$ACCOUNT_ID.json
# sed -i '' "s/$ACCOUNT_ID/transfer.$ACCOUNT_ID/" ~/.near-credentials/testnet/transfer.$ACCOUNT_ID.json
# near create-account transfer.$ACCOUNT_ID --masterAccount $ACCOUNT_ID --initialBalance 30 --publicKey "ed25519:2o5tiq68jntS8hunVjhsMfEcXnStecYf6TamQSg28ffz"
#
#
#
# cp ~/.near-credentials/testnet/transfer.$ACCOUNT_ID.json ~/.near-credentials/testnet/tf.transfer.$ACCOUNT_ID.json
# sed -i '' "s/transfer.$ACCOUNT_ID/tf.transfer.$ACCOUNT_ID/" ~/.near-credentials/testnet/tf.transfer.$ACCOUNT_ID.json
# near create-account tf.transfer.$ACCOUNT_ID --masterAccount transfer.$ACCOUNT_ID --initialBalance 10 --publicKey "ed25519:2o5tiq68jntS8hunVjhsMfEcXnStecYf6TamQSg28ffz"
# near deploy --accountId tf.transfer.$ACCOUNT_ID --initFunction 'new' --initArgs '{}' --wasmFile res/token_factory.wasm
# WASM_BYTES='cat res/wrapped_token.wasm | base64'
# near call tf.transfer.$ACCOUNT_ID store_wasm_of_token_contract $(eval "$WASM_BYTES") --base64 --accountId tf.transfer.$ACCOUNT_ID --gas 200000000000000
#
#
#
# cp ~/.near-credentials/testnet/transfer.$ACCOUNT_ID.json ~/.near-credentials/testnet/ef.transfer.$ACCOUNT_ID.json
# sed -i '' "s/transfer.$ACCOUNT_ID/ef.transfer.$ACCOUNT_ID/" ~/.near-credentials/testnet/ef.transfer.$ACCOUNT_ID.json
# near create-account ef.transfer.$ACCOUNT_ID --masterAccount transfer.$ACCOUNT_ID --initialBalance 10 --publicKey "ed25519:2o5tiq68jntS8hunVjhsMfEcXnStecYf6TamQSg28ffz"
# near deploy --accountId ef.transfer.$ACCOUNT_ID --initFunction 'new' --initArgs '{}' --wasmFile res/escrow_factory.wasm
# WASM_BYTES='cat res/channel_escrow.wasm | base64'
# near call ef.transfer.$ACCOUNT_ID store_wasm_of_channel_escrow $(eval "$WASM_BYTES") --base64 --accountId ef.transfer.$ACCOUNT_ID --gas 200000000000000
#
#
#
# near deploy --accountId $ACCOUNT_ID --initFunction 'init' --initArgs '{"appchain_registry_account":"registry.test_oct.testnet"}' --wasmFile res/near_ibc.wasm
#
#
#
# WASM_BYTES='cat res/wrapped_token.wasm | base64'
# near call 99953f25a20bceb8111198fa81c3b561.tf.transfer.$ACCOUNT_ID update_contract_code $(eval "$WASM_BYTES") --base64 --accountId tf.transfer.$ACCOUNT_ID --gas 200000000000000
#
#
#
# WASM_BYTES='cat res/channel_escrow.wasm | base64'
# near call channel-3.ef.transfer.$ACCOUNT_ID update_contract_code $(eval "$WASM_BYTES") --base64 --accountId ef.transfer.$ACCOUNT_ID --gas 200000000000000
# near call channel-3.ef.transfer.$ACCOUNT_ID migrate_state '{}' --accountId ef.transfer.$ACCOUNT_ID --gas 200000000000000
#
#
#
# near deploy --accountId ef.transfer.$ACCOUNT_ID --wasmFile res/escrow_factory.wasm
#
# near deploy --accountId tf.transfer.$ACCOUNT_ID --wasmFile res/token_factory.wasm
#
# near deploy --accountId tf.transfer.$ACCOUNT_ID --initFunction 'migrate_state' --initArgs '{}' --wasmFile res/token_factory.wasm --initGas 200000000000000
#
# near deploy --accountId $ACCOUNT_ID --wasmFile res/near_ibc.wasm
#
# near deploy --accountId $ACCOUNT_ID --initFunction 'migrate_state' --initArgs '{}' --wasmFile res/near_ibc.wasm
#
# near call $ACCOUNT_ID remove_storage_keys '{"keys":["U1RBVEU="]}' --accountId $ACCOUNT_ID --gas 200000000000000
#
# near call $ACCOUNT_ID remove_client '{"client_id":"07-tendermint-0"}' --accountId $ACCOUNT_ID --gas 200000000000000
#
# near call $ACCOUNT_ID init '' --accountId $ACCOUNT_ID --gas 200000000000000
#
# near call $ACCOUNT_ID cancel_transfer_request_in_channel_escrow '{"channel_id":"channel-10","amount":"20000000000000000000","token_denom":"OCT","sender_id":"riversyang.testnet"}' --accountId $ACCOUNT_ID --gas 200000000000000
#
# near call $ACCOUNT_ID set_max_length_of_ibc_events_history '{"max_length":100}' --accountId $ACCOUNT_ID --gas 200000000000000
#
# near call $ACCOUNT_ID register_asset_for_channel '{"channel_id":"channel-3","base_denom":"OCT","token_contract":"oct.beta_oct_relay.testnet"}' --deposit 0.01 --accountId $ACCOUNT_ID --gas 200000000000000
#
# near call $ACCOUNT_ID unregister_asset_from_channel '{"channel_id":"channel-3","base_denom":"OCT"}' --depositYocto 1 --accountId $ACCOUNT_ID --gas 200000000000000
#
