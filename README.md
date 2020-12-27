# doton-substrate

Substrate implementation for [DOTON](https://github.com/wintexpro/doton-bridge).

This repo contains two pallets:

## chainbridge

The core bridge logic. This handles voting and execution of proposals, administration of the relayer set and signaling transfers.

## simple-message-pallet

This pallet demonstrates how the chainbridge pallet can be integrated in to a substrate chain. It implements calls that can be executed through proposal only and to initiate a basic transfer across the bridge.

