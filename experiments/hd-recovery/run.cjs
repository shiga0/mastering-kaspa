// Offline disposable HD recovery experiment. Main mode generates test material.
// --worker is internal anonymous-pipe IPC; public output excludes recovery material.
'use strict';
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const digest = value => crypto.createHash('sha256').update(value).digest('hex');
const fingerprints = {
  'kaspa.js': '9a4043072f9afebb06b2c15acd7b4650707424ab95c754da9991cc9111296cf1',
  'kaspa_bg.wasm': '013ecead3156adfda76f75c6b2600d77f9c45c634b2a24db39066c1bb59fd711',
  'kaspa.d.ts': '45d1c8c7b93da4575bb0b9a49d21fab3d64be24f05b26853887f05b30bba4cc2',
};
const indices = [0, 1, 20, 21, 0x7fffffff];
const message = 'Mastering Kaspa: disposable offline HD recovery check (not a transaction)';
let phase = 'sdk-load';
function check(value) { if (!value) throw new Error('Check failed'); }
function rejected(action) {
  try { action(); } catch { return true; }
  throw new Error('Expected rejection');
}
function loadSdk() {
  check(Boolean(process.env.KASPA_SDK_ROOT));
  const root = path.join(process.env.KASPA_SDK_ROOT, 'kaspa-wasm32-sdk/nodejs/kaspa');
  check(JSON.parse(fs.readFileSync(path.join(root, 'package.json'))).version === '1.0.1');
  for (const [name, expected] of Object.entries(fingerprints)) {
    check(digest(fs.readFileSync(path.join(root, name))) === expected);
  }
  return require(root);
}
function derive(sdk, input) {
  const owned = [];
  const own = obj => { owned.push(obj); return obj; };
  try {
    phase = 'mnemonic';
    check(sdk.Mnemonic.validate(input.phrase));
    const mnemonic = own(new sdk.Mnemonic(input.phrase));
    const rawSeed = crypto.pbkdf2Sync(input.phrase, 'mnemonic' + input.passphrase, 2048, 64, 'sha512');
    const normalizedSeed = crypto.pbkdf2Sync(input.phrase.normalize('NFKD'),
      'mnemonic' + input.passphrase.normalize('NFKD'), 2048, 64, 'sha512');
    const seed = mnemonic.toSeed(input.passphrase);
    check(seed === rawSeed.toString('hex'));
    const seedMatchesNfkd = seed === normalizedSeed.toString('hex');
    check(mnemonic.toSeed(input.passphrase.normalize('NFKD')) === normalizedSeed.toString('hex'));
    rawSeed.fill(0); normalizedSeed.fill(0);
    const master = own(new sdk.XPrv(seed));
    const rows = [];
    for (const account of [0, 1]) {
      const accountPath = `m/44'/111111'/${account}'`;
      const accountKey = own(master.derivePath(accountPath));
      const accountPub = own(accountKey.toXPub());
      const watch = own(sdk.PublicKeyGenerator.fromXPub(accountPub));
      const priv = own(new sdk.PrivateKeyGenerator(master, false, BigInt(account)));
      const publicFromMaster = own(sdk.PublicKeyGenerator.fromMasterXPrv(master, false, BigInt(account)));
      for (const branch of [0, 1]) {
        for (const index of indices) {
          phase = 'derive-paths';
          const key = own(branch === 0 ? priv.receiveKey(index) : priv.changeKey(index));
          const direct = own(master.derivePath(`${accountPath}/${branch}/${index}`));
          const directKey = own(direct.toPrivateKey());
          check(key.toString() === directKey.toString());
          const publicKey = own(key.toPublicKey());
          const watchKey = own(branch === 0 ? watch.receivePubkey(index) : watch.changePubkey(index));
          const masterKey = own(branch === 0 ? publicFromMaster.receivePubkey(index) : publicFromMaster.changePubkey(index));
          check(publicKey.toString() === watchKey.toString());
          check(publicKey.toString() === masterKey.toString());
          const address = own(publicKey.toAddress('devnet'));
          const privateAddress = own(key.toAddress('devnet'));
          const mainnetAddress = own(publicKey.toAddress('mainnet'));
          const ecdsaAddress = own(publicKey.toAddressECDSA('devnet'));
          check(address.toString() === privateAddress.toString());
          check(address.toString() !== mainnetAddress.toString());
          check(address.toString() !== ecdsaAddress.toString());
          phase = 'message-signature';
          const signature = sdk.signMessage({ message, privateKey: key, noAuxRand: true });
          check(sdk.verifyMessage({ message, signature, publicKey }));
          check(!sdk.verifyMessage({ message: message + ' altered', signature, publicKey }));
          rows.push({ account, branch, index, publicKey: publicKey.toString(),
            address: address.toString(), mainnetAddress: mainnetAddress.toString(),
            ecdsaAddress: ecdsaAddress.toString(), signature });
        }
      }
    }
    phase = 'rejection-boundaries';
    const gen = own(new sdk.PrivateKeyGenerator(master, false, 0n));
    check(rejected(() => gen.receiveKey(0x80000000)));
    check(rejected(() => own(new sdk.PrivateKeyGenerator(master, false, 0x80000000n))));
    const account = own(master.derivePath("m/44'/111111'/0'"));
    const xpub = own(account.toXPub());
    check(rejected(() => own(xpub.deriveChild(0, true))));
    check(rejected(() => own(master.derivePath('m/not-an-index'))));
    const invalid = Array(12).fill('abandon').join(' ');
    check(!sdk.Mnemonic.validate(invalid));
    check(rejected(() => own(new sdk.Mnemonic(invalid))));
    return { rows, seedMatchesRawPbkdf2: true, seedMatchesNfkd,
      explicitNfkdMatchesReference: true, rejectionChecks: 6 };
  } finally {
    for (const item of owned.reverse()) item.free();
  }
}
function worker(input) {
  const run = spawnSync(process.execPath, [__filename, '--worker'], {
    input: JSON.stringify(input), encoding: 'utf8', timeout: 60_000,
    maxBuffer: 2 * 1024 * 1024,
    // Recovery material goes through an anonymous pipe; no argv/env/files.
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  if (run.status !== 0 || run.error) throw new Error('Isolated derivation failed');
  return JSON.parse(run.stdout);
}
function main(sdk) {
  const scenarios = [];
  let messageVerifications = 0;
  for (const words of [12, 24]) {
    phase = 'random-mnemonic';
    const mnemonic = sdk.Mnemonic.random(words);
    const phrase = mnemonic.phrase;
    mnemonic.free();
    const pairs = [];
    const profiles = [
      { name: 'empty', value: '', expectedNfkd: true },
      { name: 'ascii', value: 'disposable-recovery-test', expectedNfkd: true },
      { name: 'unicode-composed', value: 'caf\u00e9', expectedNfkd: false },
      { name: 'unicode-decomposed', value: 'cafe\u0301', expectedNfkd: true },
    ];
    for (const profile of profiles) {
      phase = 'isolated-recovery';
      const input = { phrase, passphrase: profile.value };
      const original = worker(input);
      // A second fresh Node process sees only the recovery material and metadata.
      const recovered = worker(input);
      check(JSON.stringify(original) === JSON.stringify(recovered));
      check(original.seedMatchesNfkd === profile.expectedNfkd);
      const addresses = original.rows.map(row => row.address);
      check(new Set(addresses).size === addresses.length);
      for (let i = 0; i < original.rows.length; i++) {
        const row = original.rows[i], restored = recovered.rows[i];
        check(sdk.verifyMessage({ message, signature: row.signature, publicKey: restored.publicKey }));
        const wrong = recovered.rows[(i + 1) % recovered.rows.length];
        check(!sdk.verifyMessage({ message, signature: row.signature, publicKey: wrong.publicKey }));
        messageVerifications++;
      }
      pairs.push(original);
      scenarios.push({ words, passphraseProfile: profile.name,
        recoveredKeys: original.rows.length, addressesUnique: true,
        samePrivateAndExplicitPath: true, accountXpubAndMasterPublicPathsMatch: true,
        networkAndAddressTypeDiffer: true, seedMatchesRawPbkdf2: true,
        seedMatchesNfkdReference: original.seedMatchesNfkd,
        explicitNfkdMatchesReference: true, originalSignaturesVerifyAfterRecovery: true,
        wrongMessageAndWrongKeyRejected: true, rejectionChecksPerProcess: original.rejectionChecks,
        publicResultsSha256: digest(JSON.stringify(original.rows)) });
    }
    for (let i = 0; i < pairs.length; i++) {
      for (let j = i + 1; j < pairs.length; j++) {
        check(pairs[i].rows.every((row, k) => row.address !== pairs[j].rows[k].address));
      }
    }
  }
  console.log(JSON.stringify({ sdkVersion: '1.0.1', allChecksPassed: true,
    scope: 'offline gen1 key/address reconstruction and message signing; no wallet discovery or transaction submission',
    mnemonicWordCounts: [12, 24], accounts: [0, 1], branches: ['receive', 'change'], indices,
    scenarioCount: scenarios.length, freshWorkerProcesses: scenarios.length * 2,
    recoveredKeys: messageVerifications, keyDerivationsAcrossBothProcesses: messageVerifications * 2,
    originalSignaturesVerifiedWithRecoveredPublicKey: messageVerifications,
    expectedUnicodeNormalizationDifferenceObserved: true,
    sdkFilesSha256: fingerprints, harnessSha256: digest(fs.readFileSync(__filename)),
    environment: { node: process.version, platform: process.platform, architecture: process.arch },
    scenarios }, null, 2));
}
try {
  const sdk = loadSdk();
  if (process.argv.includes('--worker')) {
    const input = JSON.parse(fs.readFileSync(0, 'utf8'));
    console.log(JSON.stringify(derive(sdk, input)));
  } else {
    main(sdk);
  }
} catch {
  // Never print an error object/stack: SDK/assertion errors could include secrets.
  console.error(`HD recovery experiment failed at ${phase}; recovery material not logged.`);
  process.exitCode = 1;
}
