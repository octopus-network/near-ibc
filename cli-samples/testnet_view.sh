#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export ACCOUNT_ID=v9.nearibc.testnet
#
#
#
# near view $ACCOUNT_ID version
# near view tf.transfer.$ACCOUNT_ID version
# near view ef.transfer.$ACCOUNT_ID version
# near view channel-3.ef.transfer.$ACCOUNT_ID version
#
# near view $ACCOUNT_ID get_client_consensus '{"client_id":"07-tendermint-80","consensus_height":{"revision_number":"0","revision_height":"22"}}'
#
# near view $ACCOUNT_ID get_ibc_events_heights
#
# near view $ACCOUNT_ID get_ibc_events_at '{"height":{"revision_number":0,"revision_height":142908436}}'
#
# near view $ACCOUNT_ID get_channels
#
# near view $ACCOUNT_ID get_connections
#
# near view $ACCOUNT_ID get_clients
#
# near view $ACCOUNT_ID get_next_sequence_receive '{"port_id":"provider","channel_id":"channel-7"}'
#
# near view tf.transfer.$ACCOUNT_ID get_cross_chain_assets
#
# near view tf.transfer.$ACCOUNT_ID get_cross_chain_assets '{"trace_path":"transfer/channel-221"}'
#
# near view ef.transfer.$ACCOUNT_ID get_channel_id_set
#
# near view channel-3.ef.transfer.$ACCOUNT_ID get_registered_assets
#
# near view channel-3.ef.transfer.$ACCOUNT_ID get_pending_accounts
