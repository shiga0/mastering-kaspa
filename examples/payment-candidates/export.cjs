// Export current UTXOs with the official SDK; no payment eligibility decisions.
'use strict';
const path = require('node:path');
const { setTimeout: pause } = require('node:timers/promises');

async function main() {
    const root = process.env.KASPA_SDK_ROOT;
    if (!root) throw new Error('Set KASPA_SDK_ROOT');
    globalThis.WebSocket = require(path.join(root, 'node_modules/websocket')).w3cwebsocket;
    const location = path.join(root, 'kaspa-wasm32-sdk/nodejs/kaspa');
    if (require(path.join(location, 'package.json')).version !== '1.0.1') {
        throw new Error('Expected official SDK v1.0.1');
    }
    const sdk = require(location);
    const address = 'kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99';
    const rpc = new sdk.RpcClient({ url: 'ws://127.0.0.1:18310',
        encoding: sdk.Encoding.SerdeJson, networkId: 'devnet' });
    try {
        await rpc.connect({ strategy: sdk.ConnectStrategy.Fallback });
        const info = await rpc.getServerInfo();
        if (info.networkId !== 'devnet' || info.serverVersion !== '1.0.1' || !info.hasUtxoIndex) {
            throw new Error('Expected indexed Devnet v1.0.1');
        }
        // ANCHOR: export-candidates
        const { entries } = await rpc.getUtxosByAddresses({ addresses: [address] });
        const outputs = entries.map(entry => {
            if (entry.address?.toString() !== address) throw new Error('Unexpected entry address');
            return { transaction_id: entry.outpoint.transactionId,
                output_index: entry.outpoint.index, amount_sompi: entry.amount.toString() };
        });
        console.log(JSON.stringify({ network: info.networkId, address, outputs }));
        // ANCHOR_END: export-candidates
    } finally {
        await rpc.disconnect();
    }
}

// Bound the entire one-shot export, including connect and cleanup.
const controller = new AbortController();
pause(15_000, undefined, { signal: controller.signal }).then(() => {
    console.error('SDK export timed out');
    process.exit(1);
}, () => {});
main().then(() => controller.abort(), error => {
    console.error(String(error));
    process.exit(1);
});
