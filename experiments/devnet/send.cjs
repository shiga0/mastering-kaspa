// Devnet-only experiment. Keys live only in this process and are never printed.
'use strict';
const crypto = require('node:crypto');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const runRetryChecks = process.argv.includes('--retry-checks');
const { startDropProxy } = require('./retry-transport.cjs');
const runChecks = process.argv.includes('--checks');
const sdkRoot = process.env.KASPA_SDK_ROOT;
if (!sdkRoot) throw new Error('Set KASPA_SDK_ROOT to the extracted v1.0.1 SDK directory');
globalThis.WebSocket = require(path.join(sdkRoot, 'node_modules/websocket')).w3cwebsocket;
const sdkPackage = path.join(sdkRoot, 'kaspa-wasm32-sdk/nodejs/kaspa');
if (require(path.join(sdkPackage, 'package.json')).version !== '1.0.1') {
  throw new Error('Expected official SDK release v1.0.1');
}
const sdk = require(sdkPackage);
const minerImage = 'kaspanet/cpuminer@sha256:60f78ab2828ab24b249c99210eee5a2825303a5226154260dd021ff26d46748b';
const minerName = `mastering-kaspa-send-${process.pid}`;
const deadline = Date.now() + 180_000;
const pause = () => new Promise(resolve => setTimeout(resolve, 250));
async function bounded(promise, timeoutMs = 10_000) {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => {
      timer = setTimeout(() => reject(Object.assign(new Error('RPC deadline exceeded'), { code: 'RPC_TIMEOUT' })), timeoutMs);
    })]);
  } finally { clearTimeout(timer); }
}
async function waitFor(check) {
  while (Date.now() < deadline) {
    const value = await check();
    if (value) return value;
    await pause();
  }
  throw new Error('Experiment deadline exceeded');
}
const docker = args => execFileSync('docker', args, { stdio: 'pipe', timeout: 15_000 });
let activeMiner = false;
function stopMiner() {
  if (activeMiner) { docker(['stop', '--time=2', minerName]); activeMiner = false; }
}
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => {
    try { stopMiner(); } finally { process.exit(signal === 'SIGINT' ? 130 : 143); }
  });
}
let phase = 'connect';
async function main() {
  const proxy = runRetryChecks ? await startDropProxy(sdkRoot) : null;
  const rpc = new sdk.RpcClient({ url: proxy?.url || 'ws://127.0.0.1:18310', encoding: sdk.Encoding.SerdeJson, networkId: 'devnet' });
  let minerStarted = false;
  let sourceKey, destinationKey;
  try {
    await bounded(rpc.connect());
    phase = 'server-info';
    const initial = await bounded(rpc.getServerInfo());
    if (initial.networkId !== 'devnet' || initial.serverVersion !== '1.0.1' || !initial.hasUtxoIndex) {
      throw new Error('Expected indexed Devnet v1.0.1');
    }
    phase = 'key-generation';
    sourceKey = new sdk.PrivateKey(crypto.randomBytes(32).toString('hex'));
    destinationKey = new sdk.PrivateKey(crypto.randomBytes(32).toString('hex'));
    const source = sourceKey.toKeypair().toAddress('devnet').toString();
    const destination = destinationKey.toKeypair().toAddress('devnet').toString();
    phase = 'start-miner';
    const startMiner = () => { docker(['run', '--rm', '-d', '--name', minerName, '--cpus=1', '--platform=linux/arm64',
      '--network=container:mastering-kaspa-devnet-node-1', minerImage,
      '--kaspad-address=127.0.0.1', '--port=16310', '--threads=1', '--mine-when-not-synced',
      '--mining-address=' + source]);
    minerStarted = true; activeMiner = true; };
    startMiner();
    const maturity = sdk.getNetworkParams('devnet').coinbaseTransactionMaturityPeriodDaa;
    phase = 'wait-for-mature-utxo';
    const funded = await waitFor(async () => {
      const info = await bounded(rpc.getServerInfo());
      const { entries } = await bounded(rpc.getUtxosByAddresses({ addresses: [source] }));
      const mature = entries.filter(e => info.virtualDaaScore >= e.blockDaaScore + maturity);
      return mature.length ? { entries: [mature[0]], daa: info.virtualDaaScore } : null;
    });
    if (runChecks || runRetryChecks) stopMiner();
    phase = 'build';
    const before = await bounded(rpc.getBlockDagInfo());
    const amount = 100_000_000n;
    const { transactions } = await sdk.createTransactions({ entries: funded.entries,
      outputs: [{ address: destination, amount }], priorityFee: 0n,
      changeAddress: source, networkId: 'devnet' });
    if (transactions.length !== 1) throw new Error('Expected a single transaction for this experiment');
    const pending = transactions[0];
    const inputTotal = funded.entries.reduce((sum, e) => sum + e.amount, 0n);
    const outputTotal = pending.transaction.outputs.reduce((sum, o) => sum + o.value, 0n);
    const independentFee = inputTotal - outputTotal;
    const expectedChange = inputTotal - amount - independentFee;
    if (independentFee < 0n || independentFee !== pending.feeAmount || expectedChange <= 0n) {
      throw new Error('Independent fee calculation failed');
    }
    const checks = {};
    async function rejected(label, action, pattern) {
      try { await action(); } catch (error) {
        const text = String(error);
        if (/timeout|disconnect|deadline|connect/i.test(text) || !pattern.test(text)) {
          throw new Error('Unexpected failure class in ' + label);
        }
        checks[label] = 'rejected-as-expected';
        return;
      }
      throw new Error('Unexpected acceptance in ' + label);
    }
    let competing;
    if (runChecks) {
      phase = 'insufficient-funds';
      await rejected('insufficientFunds', () => sdk.createTransactions({ entries: funded.entries,
        outputs: [{ address: destination, amount: inputTotal + 1n }], priorityFee: 0n,
        changeAddress: source, networkId: 'devnet' }), /insufficient|not enough|balance/i);
      phase = 'invalid-signature';
      await pending.sign([sourceKey]);
      phase = 'clone-invalid-signature';
      const invalid = sdk.Transaction.deserializeFromSafeJSON(pending.serializeToSafeJSON());
      phase = 'mutate-invalid-signature';
      const inputs = invalid.inputs;
      const signature = Buffer.from(inputs[0].signatureScript, 'hex');
      signature[2] ^= 1;
      inputs[0].signatureScript = signature.toString('hex');
      invalid.inputs = inputs;
      invalid.finalize();
      phase = 'submit-invalid-signature';
      await rejected('invalidSignature', () => bounded(rpc.submitTransaction({ transaction: invalid, allowOrphan: false })),
        /script|signature|validate|validation/i);
      const fresh = await bounded(rpc.getUtxosByAddresses({ addresses: [source] }));
      const original = funded.entries[0].outpoint;
      if (!fresh.entries.some(e => e.outpoint.transactionId === original.transactionId && e.outpoint.index === original.index)) {
        throw new Error('Failed signature consumed input');
      }
      checks.failedSignaturePreservedInput = true;
      phase = 'build-competing';
      const built = await sdk.createTransactions({ entries: funded.entries,
        outputs: [{ address: destination, amount: amount + 1n }], priorityFee: 0n,
        changeAddress: source, networkId: 'devnet' });
      if (built.transactions.length !== 1) throw new Error('Expected single competing transaction');
      competing = built.transactions[0];
      await competing.sign([sourceKey]);
    }
    phase = 'sign';
    await pending.sign([sourceKey]);
    phase = 'submit';
    let txid;
    if (runRetryChecks) {
      txid = pending.id;
      const signed = pending.serializeToSafeJSON();
      proxy.arm();
      try {
        await bounded(pending.submit(rpc), 500);
        throw new Error('Expected injected response loss');
      } catch (error) {
        if (error.code !== 'RPC_TIMEOUT' || proxy.stats.droppedSubmitReplies !== 1) throw error;
        checks.clientTimedOutAfterLostReply = true;
      }
      phase = 'recover-by-txid';
      await bounded(rpc.getMempoolEntry({ transactionId: txid, includeOrphanPool: false, filterTransactionPool: false }));
      checks.recoveredByTransactionId = true;
      phase = 'replay-same-transaction';
      if (pending.serializeToSafeJSON() !== signed) throw new Error('Signed bytes changed before replay');
      try {
        const reply = await bounded(rpc.submitTransaction({
          transaction: sdk.Transaction.deserializeFromSafeJSON(signed), allowOrphan: false,
        }));
        if (reply.transactionId !== txid) throw new Error('Replay returned another transaction');
        checks.sameTransactionReplay = 'same-id-returned';
      } catch (error) {
        const message = String(error);
        if (/timeout|disconnect|deadline|connect/i.test(message) || !/already.*(?:pool|exist|accepted)|duplicate/i.test(message)) throw error;
        checks.sameTransactionReplay = 'duplicate-rejected';
      }
      checks.droppedSubmitReplies = proxy.stats.droppedSubmitReplies;
      startMiner();
    } else {
      txid = await bounded(pending.submit(rpc));
    }
    if (runChecks) {
      phase = 'mempool';
      await bounded(rpc.getMempoolEntry({ transactionId: txid, includeOrphanPool: false, filterTransactionPool: false }));
      checks.mempoolObserved = true;
      phase = 'competing-spend';
      await rejected('competingSpend', () => bounded(competing.submit(rpc)), /double.spend|already.spent|already.used|orphan|missing.*outpoint|conflict/i);
      startMiner();
    }
    phase = 'receive';
    const received = await waitFor(async () => {
      const { entries } = await bounded(rpc.getUtxosByAddresses({ addresses: [destination] }));
      return entries.find(e => e.outpoint.transactionId === txid && e.amount === amount);
    });
    phase = 'acceptance';
    const accepted = await waitFor(async () => {
      const chain = await bounded(rpc.getVirtualChainFromBlock({ startHash: before.sink, includeAcceptedTransactionIds: true }));
      return chain.acceptedTransactionIds.find(b => b.acceptedTransactionIds.includes(txid));
    });
    const remaining = await bounded(rpc.getUtxosByAddresses({ addresses: [source] }));
    const input = funded.entries[0];
    if (remaining.entries.some(e => e.outpoint.transactionId === input.outpoint.transactionId && e.outpoint.index === input.outpoint.index)) {
      throw new Error('Input still appears unspent');
    }
    const changeEntries = remaining.entries.filter(e => e.outpoint.transactionId === txid);
    const actualChange = changeEntries.reduce((sum, e) => sum + e.amount, 0n);
    if (actualChange !== expectedChange) throw new Error('Observed change differs from independent calculation');
    if (runRetryChecks) {
      phase = 'duplicate-observations';
      const observed = await bounded(rpc.getUtxosByAddresses({ addresses: [destination] }));
      const credits = new Map();
      let naiveTotal = 0n;
      // Feed two identical snapshots to model repeated notification/reconciliation.
      for (const snapshot of [observed.entries, observed.entries]) {
        for (const entry of snapshot.filter(e => e.outpoint.transactionId === txid)) {
          naiveTotal += entry.amount;
          const id = `${entry.outpoint.transactionId}:${entry.outpoint.index}`;
          if (credits.has(id) && credits.get(id) !== entry.amount) throw new Error('Inconsistent duplicate');
          credits.set(id, entry.amount);
        }
      }
      const credited = [...credits.values()].reduce((sum, value) => sum + value, 0n);
      if (credited !== amount || naiveTotal !== amount * 2n || credits.size !== 1) throw new Error('Duplicate credit check failed');
      checks.naiveCreditSompi = naiveTotal.toString();
      checks.idempotentCreditSompi = credited.toString();
      checks.uniqueCreditedOutpoints = credits.size;
    }
    const result = { network: 'devnet', serverVersion: initial.serverVersion, source, destination,
      checks, inputTotalSompi: inputTotal.toString(), outputTotalSompi: outputTotal.toString(),
      independentFeeSompi: independentFee.toString(), sdkFeeSompi: pending.feeAmount.toString(),
      expectedChangeSompi: expectedChange.toString(), actualChangeSompi: actualChange.toString(),
      transactionId: txid, amountSompi: amount.toString(), coinbaseMaturityDaa: maturity.toString(),
      fundingDaaScore: funded.daa.toString(), inputSpent: true, destinationUtxoObserved: true,
      acceptedByVirtualChain: true, acceptingBlockHash: accepted.acceptingBlockHash,
      destinationOutpoint: { transactionId: received.outpoint.transactionId, index: received.outpoint.index } };
    console.log(JSON.stringify(result, null, 2));
  } finally {
    if (minerStarted) stopMiner();
    await bounded(rpc.disconnect()).catch(() => {});
    await proxy?.close();
    sourceKey?.free(); destinationKey?.free();
  }
}
main().catch(() => { console.error(`Devnet send experiment failed at ${phase}; no key material logged.`); process.exitCode = 1; });
