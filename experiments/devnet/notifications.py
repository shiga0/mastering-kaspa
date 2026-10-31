#!/usr/bin/env python3
"""Actual SDK notification, duplicate delivery, missed updates and SQLite recovery."""
import asyncio
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).parent
for name, file in [('lab', 'two-node.py'), ('storage', 'utxo-store.py')]:
    spec = importlib.util.spec_from_file_location(name, ROOT / file)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    globals()[name] = module


def compose(*args):
    subprocess.run(['docker', 'compose', '-f', str(ROOT / 'compose.yaml'), '-f', str(ROOT / 'mining.yaml'),
                    '--profile', 'mining', *args], check=True, stdout=subprocess.DEVNULL, timeout=30)


async def snapshot():
    network = await lab.rpc(18310, 'getCurrentNetwork')
    if network.get('network') != 'devnet':
        raise RuntimeError('Expected Devnet')
    return (await lab.rpc(18310, 'getUtxosByAddresses', {'addresses': [lab.ADDRESS]}))['entries']


async def main():
    watcher = None
    store = None
    try:
        compose('up', '-d', 'node')
        async def ready_snapshot():
            return {"entries": await snapshot()}
        baseline = (await lab.wait_for(ready_snapshot))["entries"]
        with tempfile.TemporaryDirectory(prefix='kaspa-utxo-store-') as directory:
            database = str(Path(directory) / 'observations.sqlite')
            store = storage.Store(database)
            store.reconcile(baseline)
            initial = store.summary()
            watcher = subprocess.Popen(['node', str(ROOT / 'watch-once.cjs')], stdout=subprocess.PIPE,
                                       stderr=subprocess.DEVNULL, text=True)
            ready = json.loads(await asyncio.wait_for(asyncio.to_thread(watcher.stdout.readline), 15))
            if not ready.get('ready'):
                raise RuntimeError('Watcher did not subscribe')
            compose('up', '-d', 'miner')
            message = json.loads(await asyncio.wait_for(asyncio.to_thread(watcher.stdout.readline), 30))
            event = message['event']
            compose('stop', '-t', '2', 'miner')
            await asyncio.to_thread(watcher.wait, 10)
            if watcher.returncode:
                raise RuntimeError('Watcher failed')
            # The SDK uses UtxoEntryReference objects serialized to a public JSON schema.
            store.apply(event['added'], event['removed'])
            once = store.summary()
            store.apply(event['added'], event['removed'])
            duplicate = store.summary()
            if once != duplicate or not event['added']:
                raise RuntimeError('Duplicate changed stored amount or event had no additions')
            before = await lab.rpc(18310, 'getBlockDagInfo')
            compose('up', '-d', 'miner')  # Subscriber has closed: these notifications are missed.
            async def advanced():
                dag = await lab.rpc(18310, 'getBlockDagInfo')
                return dag if dag['blockCount'] >= before['blockCount'] + 5 else None
            await lab.wait_for(advanced)
            compose('stop', '-t', '2', 'miner')
            full = await snapshot()
            if store.summary()['activeOutpoints'] >= len(full):
                raise RuntimeError('Expected missing additions while unsubscribed')
            store.close(); store = None
            reopened = json.loads(subprocess.check_output([sys.executable, str(ROOT / 'utxo-store.py'), database], text=True))
            if reopened != duplicate:
                raise RuntimeError('Committed observations did not survive process reopen')
            store = storage.Store(database)
            store.reconcile(full)
            recovered = store.summary()
            expected = {'activeOutpoints': len(full), 'activeAmountSompi': str(sum(int(e['utxoEntry']['amount']) for e in full))}
            if recovered != expected:
                raise RuntimeError('Reconciliation mismatch')
            store.reconcile(full)
            if store.summary() != recovered:
                raise RuntimeError('Repeated snapshot changed state')
            print(json.dumps({'network':'devnet', 'initial':initial, 'notificationAdded':len(event['added']),
                              'notificationRemoved':len(event['removed']), 'afterOneEvent':once,
                              'afterDuplicateEvent':duplicate, 'processReopen':reopened,
                              'reconciled':recovered, 'matchedRpcSnapshot':True,
                              'repeatedSnapshotWasIdempotent':True, 'databasePublished':False}, indent=2))
    finally:
        if watcher and watcher.poll() is None:
            watcher.terminate()
            watcher.wait(timeout=10)
        if store:
            store.close()
        compose('stop', '-t', '2')


if __name__ == '__main__':
    asyncio.run(main())
