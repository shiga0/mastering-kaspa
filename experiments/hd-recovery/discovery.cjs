// Official Wallet.accountsDiscovery against a synthetic, localhost-only UTXO RPC.
'use strict';
const fs = require('node:fs'), path = require('node:path'), crypto = require('node:crypto');
const { spawnSync } = require('node:child_process');
const { verifyModel } = require('./discovery-model.cjs');
const digest = x => crypto.createHash('sha256').update(x).digest('hex');
const check = x => { if (!x) throw Error('Discovery comparison failed'); };
const sdkFiles = {
  'kaspa.js': '9a4043072f9afebb06b2c15acd7b4650707424ab95c754da9991cc9111296cf1',
  'kaspa_bg.wasm': '013ecead3156adfda76f75c6b2600d77f9c45c634b2a24db39066c1bb59fd711',
  'kaspa.d.ts': '45d1c8c7b93da4575bb0b9a49d21fab3d64be24f05b26853887f05b30bba4cc2',
};
let phase = 'setup';
function rawId(frame) {
  const match = frame.match(/"id"\s*:\s*(\d+)/);
  check(match);
  return match[1];
}
async function worker() {
  const root = process.env.KASPA_SDK_ROOT;
  check(root);
  const sdkRoot = path.join(root, 'kaspa-wasm32-sdk/nodejs/kaspa');
  for (const [file, hash] of Object.entries(sdkFiles)) check(digest(fs.readFileSync(path.join(sdkRoot, file))) === hash);
  const websocket = require(path.join(root, 'node_modules/websocket'));
  globalThis.WebSocket = websocket.w3cwebsocket;
  const sdk = require(sdkRoot);
  const vectors = JSON.parse(fs.readFileSync(path.join(__dirname, 'discovery-vectors.json')));
  const mnemonic = sdk.Mnemonic.random(12), phrase = mnemonic.phrase;
  const master = new sdk.XPrv(mnemonic.toSeed());
  const byAddress = new Map();
  try {
    for (let account = 0; account < 5; account++) {
      const generator = sdk.PublicKeyGenerator.fromMasterXPrv(master, false, BigInt(account));
      try {
        for (const branch of [0, 1]) for (let index = 0; index < 22; index++) {
          const address = branch === 0 ? generator.receiveAddress('devnet', index) : generator.changeAddress('devnet', index);
          const script = sdk.payToAddressScript(address);
          byAddress.set(address.toString(), { account, branch, index,
            scriptPublicKey: { version: script.version, script: script.script } });
          script.free(); address.free();
        }
      } finally { generator.free(); }
    }
  } finally { master.free(); mnemonic.free(); }
  // Demonstrate why JSON.parse/stringify alone cannot echo all SDK request IDs.
  const sample = '{"id":9007199254740993}';
  check(rawId(sample) === '9007199254740993' && String(JSON.parse(sample).id) !== rawId(sample));
  const http = require('node:http'), server = http.createServer();
  const wss = new websocket.server({ httpServer: server });
  const connections = new Set();
  let active, requests, serverFailed = false, largeIds = 0;
  wss.on('request', request => {
    const connection = request.accept(null, request.origin);
    connections.add(connection);
    connection.on('close', () => connections.delete(connection));
    connection.on('error', () => { serverFailed = true; });
    connection.on('message', message => {
      try {
        check(message.type === 'utf8' && active);
        const frame = message.utf8Data, request = JSON.parse(frame), id = rawId(frame);
        check(request.method === 'getUtxosByAddresses' && requests.length < 10);
        if (BigInt(id) > BigInt(Number.MAX_SAFE_INTEGER)) largeIds++;
        const addresses = request.params.addresses;
        check(Array.isArray(addresses));
        const coordinates = addresses.map(address => { const c = byAddress.get(address); check(c); return c; });
        if (coordinates.length) {
          check(new Set(coordinates.map(c => c.account)).size === 1);
          check(coordinates.length === active.addressExtent * 2);
          check(new Set(addresses).size === addresses.length);
          const actual = coordinates.map(c => `${c.branch}:${c.index}`).sort();
          const expected = [0, 1].flatMap(branch => Array.from({ length: active.addressExtent }, (_, index) => `${branch}:${index}`)).sort();
          check(JSON.stringify(actual) === JSON.stringify(expected));
        }
        const account = coordinates.length ? coordinates[0].account : null;
        const entries = addresses.flatMap(address => {
          const c = byAddress.get(address);
          if (!active.utxos.some(([a, b, i]) => a === c.account && b === c.branch && i === c.index)) return [];
          return [{ address, outpoint: { transactionId: '11'.repeat(32), index: c.index },
            utxoEntry: { amount: 100000000, scriptPublicKey: c.scriptPublicKey, blockDaaScore: 1, isCoinbase: false } }];
        });
        requests.push({ account, addressCount: addresses.length, matchedUtxos: entries.length });
        // Keep the original u64 token, rather than the rounded JavaScript Number.
        connection.sendUTF('{"id":' + id + ',"method":"getUtxosByAddresses","params":' + JSON.stringify({ entries }) + '}');
      } catch { serverFailed = true; connection.drop(); }
    });
  });
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const results = [];
  try {
    const url = `ws://127.0.0.1:${server.address().port}`;
    for (const vector of vectors) {
      phase = 'official-' + vector.name;
      active = vector; requests = [];
      const wallet = new sdk.Wallet({ resident: true, networkId: 'devnet', encoding: sdk.Encoding.SerdeJson, url });
      try {
        await wallet.connect({ url, blockAsyncConnect: true, timeoutDuration: 3000 });
        const response = await wallet.accountsDiscovery({ discoveryKind: sdk.AccountsDiscoveryKind.Bip44,
          accountScanExtent: vector.accountExtent, addressScanExtent: vector.addressExtent, bip39_mnemonic: phrase });
        check(!serverFailed && response.lastAccountIndexFound === vector.expectedLast);
        check(JSON.stringify(requests.map(r => r.account)) === JSON.stringify(vector.expectedAccounts));
        results.push({ name: vector.name, lastAccountIndexFound: response.lastAccountIndexFound,
          addressExtent: vector.addressExtent, accountExtent: vector.accountExtent,
          accountsQueried: requests.map(r => r.account), addressesQueried: requests.reduce((n, r) => n + r.addressCount, 0),
          matchedUtxos: requests.reduce((n, r) => n + r.matchedUtxos, 0),
          historicalUseNotAvailableThroughThisRpc: Boolean(vector.historicalUse), comparisonMatched: true });
      } finally { await wallet.disconnect(); wallet.free(); }
    }
  } finally {
    for (const connection of connections) connection.drop();
    wss.shutDown();
    await new Promise(resolve => server.close(resolve));
  }
  check(largeIds > 0);
  const files = ['discovery.cjs', 'discovery-model.cjs', 'discovery-vectors.json'];
  const output = { sdkVersion: '1.0.1', officialCaseCount: results.length, allComparisonsMatched: true,
    scope: 'official Wallet.accountsDiscovery with synthetic UTXO RPC; educational gap model separately',
    residentWallet: true, rpcServerClosed: true, largeRequestIdsPreserved: largeIds,
    sdkFilesSha256: sdkFiles, filesSha256: Object.fromEntries(files.map(file => [file, digest(fs.readFileSync(path.join(__dirname, file)))])),
    environment: { node: process.version, platform: process.platform, architecture: process.arch },
    officialResults: results, educationalGapModel: verifyModel() };
  console.log('BOOK_DISCOVERY_RESULT ' + JSON.stringify(output));
}
if (process.argv.includes('--worker')) {
  setTimeout(() => { console.error('Discovery worker deadline'); process.exit(2); }, 45_000).unref();
  worker().then(() => process.exit(0)).catch(() => {
    console.error(`Discovery failed at ${phase}; recovery material not logged.`); process.exit(1);
  });
} else {
  const run = spawnSync(process.execPath, [__filename, '--worker'], { encoding: 'utf8', timeout: 60_000,
    maxBuffer: 2 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  const lines = (run.stdout || '').split('\n').filter(line => line.startsWith('BOOK_DISCOVERY_RESULT '));
  if (run.status !== 0 || lines.length !== 1) {
    // Only expose our controlled stage name, never raw SDK logs/errors.
    const stage = (run.stderr || '').match(/Discovery failed at ([a-z0-9-]+);/);
    console.error('Discovery experiment failed' + (stage ? ' at ' + stage[1] : '') + '; raw worker output not published.');
    process.exitCode = 1;
  } else console.log(JSON.stringify(JSON.parse(lines[0].slice('BOOK_DISCOVERY_RESULT '.length)), null, 2));
}
