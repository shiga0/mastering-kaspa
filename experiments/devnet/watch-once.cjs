// Receive one actual UTXO notification using the official v1.0.1 SDK.
'use strict';
const path = require('node:path');
const root = process.env.KASPA_SDK_ROOT;
if (!root) throw new Error('Set KASPA_SDK_ROOT');
globalThis.WebSocket = require(path.join(root, 'node_modules/websocket')).w3cwebsocket;
const location = path.join(root, 'kaspa-wasm32-sdk/nodejs/kaspa');
if (require(path.join(location, 'package.json')).version !== '1.0.1') throw new Error('Expected SDK 1.0.1');
const sdk = require(location);
const address = process.env.KASPA_WATCH_ADDRESS || 'kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99';
const url = process.env.KASPA_RPC_URL || 'ws://127.0.0.1:18310';
if (!['ws://127.0.0.1:18310', 'ws://127.0.0.1:19310', 'ws://127.0.0.1:19320'].includes(url)) throw new Error('Only local experiment RPC endpoints are allowed');
const rpc = new sdk.RpcClient({url, encoding:sdk.Encoding.SerdeJson, networkId:'devnet'});
const stream = process.argv.includes('--stream');
const hintsOnly = process.argv.includes('--hints');
let done = false;
const timer = setTimeout(() => { console.error('Notification deadline exceeded'); process.exit(1); }, stream ? 120_000 : 60_000);
(async () => {
  await rpc.connect();
  const info = await rpc.getServerInfo();
  if (info.networkId !== 'devnet' || info.serverVersion !== '1.0.1') throw new Error('Wrong network/version');
  rpc.addEventListener('utxos-changed', async event => {
    if (hintsOnly) { console.log(JSON.stringify({changed: true})); return; }
    if (done) return;
    if (!stream) done = true;
    const changes = event.data ?? event.detail;
    const entry = value => ({
      outpoint: { transactionId: value.outpoint.transactionId, index: value.outpoint.index },
      utxoEntry: { amount: value.amount.toString() },
    });
    console.log(JSON.stringify({event: {added: changes.added.map(entry), removed: changes.removed.map(entry)}}));
    if (!stream) { await rpc.disconnect(); clearTimeout(timer); process.exit(0); }
  });
  await rpc.subscribeUtxosChanged([address]);
  console.log(JSON.stringify({ready: true}));
})().catch(() => { console.error('Notification subscription failed'); process.exit(1); });
