// Official v1.0.1 Node.js/WASM SDK: read-only, local Devnet demonstration.
'use strict';
const path = require('node:path');
const { setTimeout: pause } = require('node:timers/promises');

const address = 'kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99';
const url = 'ws://127.0.0.1:18310';
const log = value => console.log(JSON.stringify(value));

async function bounded(operation, label, milliseconds = 10_000) {
    let timer;
    try {
        return await Promise.race([
            operation,
            new Promise((_, reject) => {
                timer = setTimeout(() => reject(new Error(`${label}: timed out`)), milliseconds);
            }),
        ]);
    } finally {
        clearTimeout(timer);
    }
}

async function session(sdk, number) {
    const rpc = new sdk.RpcClient({ url, encoding: sdk.Encoding.SerdeJson, networkId: 'devnet' });
    // Register before subscribing so an early notification is not lost locally.
    let received;
    const firstChange = new Promise(resolve => { received = resolve; });
    const onChange = event => {
        const change = event.data ?? event.detail;
        received({ added: change.added.length, removed: change.removed.length });
    };
    rpc.addEventListener('utxos-changed', onChange);
    try {
        // ANCHOR: connect-and-observe
        await bounded(rpc.connect({ strategy: sdk.ConnectStrategy.Fallback }), 'connect');
        const info = await bounded(rpc.getServerInfo(), 'getServerInfo');
        if (info.networkId !== 'devnet' || info.serverVersion !== '1.0.1' || !info.hasUtxoIndex) {
            throw new Error('Expected indexed Devnet v1.0.1');
        }
        log({ step: 'connected', session: number, network: info.networkId,
            serverVersion: info.serverVersion, isSynced: info.isSynced });

        // Each new connection needs a subscription and a fresh snapshot.
        await bounded(rpc.subscribeUtxosChanged([address]), 'subscribeUtxosChanged');
        async function snapshot(reason) {
            const { entries } = await bounded(
                rpc.getUtxosByAddresses({ addresses: [address] }), 'getUtxosByAddresses');
            const amount = entries.reduce((sum, entry) => sum + entry.amount, 0n);
            log({ step: 'snapshot', session: number, reason, utxoCount: entries.length,
                observedAmountSompi: amount.toString() });
        }
        await snapshot(number === 1 ? 'initial' : 'after-reconnect');
        log({ step: 'subscribed', session: number });
        const counts = await bounded(firstChange, 'wait for UTXO notification; start the miner', 60_000);
        log({ step: 'notification', session: number, ...counts });
        // A notification is a hint to query current state, not a payment receipt.
        await snapshot('after-notification');
        // ANCHOR_END: connect-and-observe
    } finally {
        rpc.removeEventListener('utxos-changed', onChange);
        await bounded(rpc.disconnect(), 'disconnect');
        log({ step: 'disconnected', session: number });
    }
}

async function main() {
    const root = process.env.KASPA_SDK_ROOT;
    if (!root) throw new Error('Set KASPA_SDK_ROOT; see examples/sdk-monitor/README.md');
    globalThis.WebSocket = require(path.join(root, 'node_modules/websocket')).w3cwebsocket;
    const location = path.join(root, 'kaspa-wasm32-sdk/nodejs/kaspa');
    if (require(path.join(location, 'package.json')).version !== '1.0.1') {
        throw new Error('Expected official SDK v1.0.1');
    }
    const sdk = require(location);
    await session(sdk, 1);
    log({ step: 'offline-gap', milliseconds: 2_000 });
    await pause(2_000);
    await session(sdk, 2);
    log({ step: 'complete', sessions: 2 });
}

main().catch(error => {
    console.error(String(error));
    // A timeout bounds waiting; it does not cancel an outstanding SDK operation.
    // Exit after session cleanup so SDK background tasks cannot keep a failed demo alive.
    process.exit(1);
});
