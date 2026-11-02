// Disposable gen1 Devnet recovery. Main mode creates all recovery material internally.
'use strict';
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { execFileSync, spawnSync } = require('node:child_process');
const hash = value => crypto.createHash('sha256').update(value).digest('hex');
const sdkFiles = {
  'kaspa.js': '9a4043072f9afebb06b2c15acd7b4650707424ab95c754da9991cc9111296cf1',
  'kaspa_bg.wasm': '013ecead3156adfda76f75c6b2600d77f9c45c634b2a24db39066c1bb59fd711',
  'kaspa.d.ts': '45d1c8c7b93da4575bb0b9a49d21fab3d64be24f05b26853887f05b30bba4cc2',
};
const minerImage = 'kaspanet/cpuminer@sha256:60f78ab2828ab24b249c99210eee5a2825303a5226154260dd021ff26d46748b';
const nodeName = 'mastering-kaspa-devnet-node-1';
const minerName = `mastering-kaspa-hd-send-${process.pid}`;
const paths = { receive: "m/44'/111111'/1'/0/21", change: "m/44'/111111'/1'/1/20", destination: "m/44'/111111'/0'/0/0" };
let phase = 'sdk-load';
function check(value) { if (!value) throw new Error('Check failed'); }
function loadSdk() {
  check(Boolean(process.env.KASPA_SDK_ROOT));
  const root = path.join(process.env.KASPA_SDK_ROOT, 'kaspa-wasm32-sdk/nodejs/kaspa');
  check(JSON.parse(fs.readFileSync(path.join(root, 'package.json'))).version === '1.0.1');
  for (const [name, digest] of Object.entries(sdkFiles)) check(hash(fs.readFileSync(path.join(root, name))) === digest);
  return require(root);
}
function worker(sdk, input) {
  const owned = [], own = x => { owned.push(x); return x; };
  try {
    phase = 'restore';
    // ANCHOR: restore-hd-keys
    check(sdk.Mnemonic.validate(input.phrase));
    const mnemonic = own(new sdk.Mnemonic(input.phrase));
    const root = own(new sdk.XPrv(mnemonic.toSeed(input.passphrase)));
    const keys = {}, addresses = {};
    for (const [name, derivation] of Object.entries(paths)) {
      const child = own(root.derivePath(derivation));
      keys[name] = own(child.toPrivateKey());
      addresses[name] = own(keys[name].toAddress('devnet')).toString();
    }
    // ANCHOR_END: restore-hd-keys
    if (input.mode === 'addresses') return { addresses };
    phase = 'address-match-before-signing';
    // ANCHOR: sign-restored-transaction
    check(JSON.stringify(addresses) === JSON.stringify(input.expectedAddresses));
    phase = 'sign-restored-transaction';
    check(input.branch === 'receive' || input.branch === 'change');
    const transaction = own(sdk.Transaction.deserializeFromSafeJSON(input.unsigned));
    // Local signature verification is requested before returning public signed bytes.
    const signed = own(sdk.signTransaction(transaction, [keys[input.branch]], true));
    signed.finalize();
    return { addresses, signed: signed.serializeToSafeJSON(), localSignatureVerified: true };
    // ANCHOR_END: sign-restored-transaction
  } finally { for (const item of owned.reverse()) item.free(); }
}
function invoke(input, expectAddressMismatch = false) {
  const result = spawnSync(process.execPath, [__filename, '--worker'], {
    input: JSON.stringify(input), encoding: 'utf8', timeout: 30_000,
    maxBuffer: 2 * 1024 * 1024, stdio: ['pipe', 'pipe', 'pipe'],
  });
  if (expectAddressMismatch) {
    check(result.status === 1 && result.stdout === '' &&
      result.stderr.trim() === 'HD Devnet experiment failed at address-match-before-signing; recovery material not logged.');
    return;
  }
  check(result.status === 0 && !result.error);
  return JSON.parse(result.stdout);
}
const docker = args => execFileSync('docker', args, { stdio: 'pipe', timeout: 20_000 }).toString().trim();
const compose = ['compose', '-f', 'experiments/devnet/compose.yaml', '-f', 'experiments/devnet/mining.yaml', '-f', 'experiments/devnet/fresh.yaml'];
let minerAttempted = false, nodeManaged = false;
function stopMiner() {
  if (minerAttempted) {
    if (docker(['ps', '-a', '--filter', `name=^/${minerName}$`, '--format', '{{.Names}}'])) docker(['stop', '--time=2', minerName]);
    minerAttempted = false;
  }
}
function cleanup() {
  try { stopMiner(); }
  finally { if (nodeManaged) { docker([...compose, 'stop', '-t', '2', 'node']); nodeManaged = false; } }
}
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => {
  try { cleanup(); } finally { process.exit(signal === 'SIGINT' ? 130 : 143); }
});
const pause = () => new Promise(resolve => setTimeout(resolve, 250));
async function bounded(promise) {
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('RPC deadline')), 10_000); })]); }
  finally { clearTimeout(timer); }
}
async function main(sdk) {
  const deadline = Date.now() + 240_000;
  async function waitFor(fn) {
    while (Date.now() < deadline) { const value = await fn(); if (value) return value; await pause(); }
    throw new Error('Experiment deadline');
  }
  const startMiner = address => {
    minerAttempted = true;
    docker(['run', '--rm', '-d', '--name', minerName, '--cpus=1', '--platform=linux/arm64',
      '--network=container:' + nodeName, minerImage, '--kaspad-address=127.0.0.1', '--port=16310',
      '--threads=1', '--mine-when-not-synced', '--mining-address=' + address]);
  };
  let rpc;
  const transactions = [];
  let result;
  try {
    phase = 'preflight';
    check(Boolean(process.env.KASPA_FRESH_VOLUME));
    // Refuse to take over a running single-node exercise.
    check(docker(['ps', '--filter', 'name=mastering-kaspa-devnet-', '--format', '{{.Names}}']) === '');
    const mnemonic = sdk.Mnemonic.random(24);
    const phrase = mnemonic.phrase;
    mnemonic.free();
    const recovery = { phrase, passphrase: 'disposable-hd-devnet-check' };
    phase = 'original-address-worker';
    const { addresses } = invoke({ ...recovery, mode: 'addresses' });
    phase = 'start-node';
    nodeManaged = true;
    docker([...compose, 'up', '-d', '--force-recreate', 'node']);
    const mounts = JSON.parse(docker(['inspect', nodeName, '--format', '{{json .Mounts}}']));
    check(mounts.some(m => m.Destination === '/data' && m.Name === process.env.KASPA_FRESH_VOLUME));
    phase = 'rpc-readiness';
    globalThis.WebSocket = require(path.join(process.env.KASPA_SDK_ROOT, 'node_modules/websocket')).w3cwebsocket;
    rpc = new sdk.RpcClient({ url: 'ws://127.0.0.1:18310', encoding: sdk.Encoding.SerdeJson, networkId: 'devnet' });
    await bounded(rpc.connect());
    const info = await bounded(rpc.getServerInfo());
    check(info.networkId === 'devnet' && info.serverVersion === '1.0.1' && info.hasUtxoIndex);
    const maturity = sdk.getNetworkParams('devnet').coinbaseTransactionMaturityPeriodDaa;
    phase = 'fund-receive-address';
    startMiner(addresses.receive);
    const funding = await waitFor(async () => {
      const state = await bounded(rpc.getServerInfo());
      const { entries } = await bounded(rpc.getUtxosByAddresses({ addresses: [addresses.receive] }));
      const mature = entries.find(e => state.virtualDaaScore >= e.blockDaaScore + maturity);
      return mature ? { entry: mature, daa: state.virtualDaaScore } : null;
    });
    stopMiner();
    let entry = funding.entry;
    for (const branch of ['receive', 'change']) {
      phase = 'build-' + branch;
      const amount = 100_000_000n;
      const before = await bounded(rpc.getBlockDagInfo());
      const { transactions: built } = await sdk.createTransactions({ entries: [entry],
        outputs: [{ address: addresses.destination, amount }], priorityFee: 0n,
        changeAddress: addresses.change, networkId: 'devnet' });
      check(built.length === 1);
      const pending = built[0];
      const inputTotal = entry.amount;
      const outputTotal = pending.transaction.outputs.reduce((n, output) => n + output.value, 0n);
      const fee = inputTotal - outputTotal;
      const expectedChange = inputTotal - amount - fee;
      check(fee === pending.feeAmount && fee >= 0n && expectedChange > 0n);
      const unsigned = pending.serializeToSafeJSON();
      if (branch === 'receive') {
        phase = 'wrong-passphrase-before-signing';
        invoke({ ...recovery, passphrase: recovery.passphrase + '-wrong', mode: 'sign', branch,
          expectedAddresses: addresses, unsigned }, true);
      }
      phase = 'recovered-signing-worker-' + branch;
      const restored = invoke({ ...recovery, mode: 'sign', branch, expectedAddresses: addresses, unsigned });
      check(restored.localSignatureVerified);
      const signed = sdk.Transaction.deserializeFromSafeJSON(restored.signed);
      check(signed.id === pending.id);
      const stripSignatures = text => {
        const data = JSON.parse(text);
        for (const input of data.inputs) delete input.signatureScript;
        return JSON.stringify(data);
      };
      check(stripSignatures(unsigned) === stripSignatures(restored.signed));
      phase = 'submit-' + branch;
      const reply = await bounded(rpc.submitTransaction({ transaction: signed, allowOrphan: false }));
      check(reply.transactionId === signed.id);
      const txid = reply.transactionId;
      await bounded(rpc.getMempoolEntry({ transactionId: txid, includeOrphanPool: false, filterTransactionPool: false }));
      startMiner(addresses.receive);
      phase = 'virtual-acceptance-' + branch;
      const accepted = await waitFor(async () => {
        const chain = await bounded(rpc.getVirtualChainFromBlock({ startHash: before.sink, includeAcceptedTransactionIds: true }));
        return chain.acceptedTransactionIds.find(b => b.acceptedTransactionIds.includes(txid));
      });
      const received = await waitFor(async () => {
        const state = await bounded(rpc.getUtxosByAddresses({ addresses: [addresses.destination] }));
        return state.entries.find(e => e.outpoint.transactionId === txid && e.amount === amount);
      });
      const change = await waitFor(async () => {
        const state = await bounded(rpc.getUtxosByAddresses({ addresses: [addresses.change] }));
        return state.entries.find(e => e.outpoint.transactionId === txid && e.amount === expectedChange);
      });
      const old = await bounded(rpc.getUtxosByAddresses({ addresses: [addresses[branch]] }));
      check(!old.entries.some(e => e.outpoint.transactionId === entry.outpoint.transactionId && e.outpoint.index === entry.outpoint.index));
      stopMiner();
      transactions.push({ branch, transactionId: txid, inputOutpoint: { transactionId: entry.outpoint.transactionId, index: entry.outpoint.index },
        inputTotalSompi: inputTotal.toString(), amountSompi: amount.toString(), feeSompi: fee.toString(),
        sdkFeeSompi: pending.feeAmount.toString(), expectedChangeSompi: expectedChange.toString(),
        actualChangeSompi: change.amount.toString(), destinationOutpoint: { transactionId: received.outpoint.transactionId, index: received.outpoint.index },
        changeOutpoint: { transactionId: change.outpoint.transactionId, index: change.outpoint.index },
        acceptingBlockHash: accepted.acceptingBlockHash, inputSpent: true, mempoolObserved: true,
        acceptedByVirtualChain: true, localSignatureVerifiedAfterRecovery: true, unsignedFieldsPreserved: true });
      entry = change;
      pending.free(); signed.free();
    }
    check(transactions[1].inputOutpoint.transactionId === transactions[0].changeOutpoint.transactionId &&
      transactions[1].inputOutpoint.index === transactions[0].changeOutpoint.index);
    result = { network: 'devnet', sdkVersion: '1.0.1', serverVersion: info.serverVersion,
      paths, addresses, mnemonicWords: 24, passphraseProfile: 'ascii', coinbaseMaturityDaa: maturity.toString(),
      fundingDaaScore: funding.daa.toString(), originalWorkerExitedBeforeFunding: true,
      recoveredSigningProcesses: 2, wrongPassphraseRejectedBeforeSigning: true,
      recoveredChangeSpentAgain: true, transactions, sdkFilesSha256: sdkFiles,
      harnessSha256: hash(fs.readFileSync(__filename)), environment: { node: process.version, platform: process.platform, architecture: process.arch } };
  } finally {
    try { if (rpc) await bounded(rpc.disconnect()).catch(() => {}); }
    finally { cleanup(); }
  }
  phase = 'stopped-state';
  check(docker(['inspect', nodeName, '--format', '{{.State.Running}}']) === 'false');
  check(docker(['ps', '--filter', `name=^/${minerName}$`, '--format', '{{.Names}}']) === '');
  result.nodeStopped = true; result.minerStopped = true;
  console.log(JSON.stringify(result, null, 2));
}
(async () => {
  const sdk = loadSdk();
  if (process.argv.includes('--worker')) console.log(JSON.stringify(worker(sdk, JSON.parse(fs.readFileSync(0, 'utf8')))));
  else await main(sdk);
})().catch(() => {
  console.error(`HD Devnet experiment failed at ${phase}; recovery material not logged.`);
  process.exitCode = 1;
});
