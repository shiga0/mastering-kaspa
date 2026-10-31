#!/usr/bin/env python3
"""Observe reward UTXO removals after merging independently mined Devnet branches."""
import asyncio
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).parent
for name, filename in [('partition', 'partition.py'), ('storage', 'utxo-store.py')]:
    spec = importlib.util.spec_from_file_location(name, ROOT / filename)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    globals()[name] = module
lab = partition.lab
B_ADDRESS = 'kaspadev:qr2tqghdk4tkd36m2squjmch2sdgf3w2tpr3uce3zc2xreknm0z4xhgvjnq8v'


def compose(*args):
    subprocess.run(['docker', 'compose', '-p', 'mastering-kaspa-reorg-small', '-f', str(ROOT / 'two-node.yaml'),
                    '-f', str(ROOT / 'partition.yaml'), '-f', str(ROOT / 'reorg.yaml'),
                    '--profile', 'mining', *args], check=True,
                   stdout=subprocess.DEVNULL, timeout=30)


def container_ip(service):
    args = ['docker', 'compose', '-p', 'mastering-kaspa-reorg-small', '-f', str(ROOT / 'two-node.yaml'),
            '-f', str(ROOT / 'partition.yaml'), '-f', str(ROOT / 'reorg.yaml')]
    cid = subprocess.check_output([*args, 'ps', '-q', service], text=True).strip()
    meta = json.loads(subprocess.check_output(['docker', 'inspect', cid], text=True))[0]
    return next(iter(meta['NetworkSettings']['Networks'].values()))['IPAddress']


partition.container_ip = container_ip
partition.compose = compose


async def snapshot(port):
    if (await lab.rpc(port, 'getCurrentNetwork'))['network'] != 'devnet':
        raise RuntimeError('Expected Devnet')
    return (await lab.rpc(port, 'getUtxosByAddresses', {'addresses': [B_ADDRESS]}))['entries']


def keyed(entries):
    return {(e['outpoint']['transactionId'], e['outpoint']['index']): int(e['utxoEntry']['amount']) for e in entries}


async def main():
    watcher = reader = None
    store = None
    try:
        partition.compose('up', '-d', 'node-a', 'relay')
        os.environ['KASPA_LAB_RELAY_IP'] = partition.container_ip('relay')
        partition.compose('up', '-d', 'node-b')
        async def ready_both():
            return await asyncio.gather(lab.state(19310), lab.state(19320))
        initial = await lab.wait_for(ready_both)
        # Fresh empty nodes need one common mined block before partitioning.
        if initial[0]['blockCount'] == 0:
            partition.compose('up', '-d', '--no-deps', 'miner')
            async def common_progress():
                return (await lab.state(19310))['blockCount'] >= 3
            await lab.wait_for(common_progress)
            partition.compose('stop', '-t', '2', 'miner')
        baseline = await lab.wait_for(lab.equal_states)
        partition.compose('stop', '-t', '2', 'relay')
        async def disconnected():
            return await partition.peer_count(19310) == 0 and await partition.peer_count(19320) == 0
        await lab.wait_for(disconnected)
        # B's short fork first; A then builds a longer alternative while disconnected.
        partition.compose('up', '-d', '--no-deps', 'miner-b')
        async def b_advanced():
            b = await lab.state(19320)
            return b if b['blockCount'] >= baseline['blockCount'] + 3 else None
        await lab.wait_for(b_advanced)
        partition.compose('stop', '-t', '2', 'miner-b')
        before_b = await lab.state(19320)
        before_entries = await snapshot(19320)
        partition.compose('up', '-d', '--no-deps', 'miner')
        async def a_advanced():
            a = await lab.state(19310)
            return a if a['blockCount'] >= before_b['blockCount'] + 5 else None
        await lab.wait_for(a_advanced)
        partition.compose('stop', '-t', '2', 'miner')
        before_a = await lab.state(19310)
        if before_a['sink'] == before_b['sink']:
            raise RuntimeError('Branches did not diverge')
        removed_notifications = set()
        notification_count = 0
        watcher = await asyncio.create_subprocess_exec(
            'node', str(ROOT / 'watch-once.cjs'), '--stream',
            env={**os.environ, 'KASPA_RPC_URL': 'ws://127.0.0.1:19320', 'KASPA_WATCH_ADDRESS': B_ADDRESS},
            stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.DEVNULL)
        ready = json.loads(await asyncio.wait_for(watcher.stdout.readline(), 15))
        if not ready.get('ready'):
            raise RuntimeError('Watcher did not subscribe')
        async def consume():
            nonlocal notification_count
            while line := await watcher.stdout.readline():
                event = json.loads(line)['event']
                notification_count += 1
                for entry in event['removed']:
                    point = entry['outpoint']
                    removed_notifications.add((point['transactionId'], point['index']))
        reader = asyncio.create_task(consume())
        with tempfile.TemporaryDirectory(prefix='kaspa-reorg-observations-') as directory:
            database = str(Path(directory) / 'observations.sqlite')
            store = storage.Store(database)
            store.reconcile(before_entries)
            prior = store.summary()
            partition.compose('up', '-d', '--no-deps', 'relay')
            if partition.container_ip('relay') != os.environ['KASPA_LAB_RELAY_IP']:
                raise RuntimeError('Relay IP changed')
            async def reconnected():
                return await partition.peer_count(19310) > 0 and await partition.peer_count(19320) > 0
            await lab.wait_for(reconnected)
            # Mine a joining block after both branches have been exchanged.
            async def branches_known():
                a, b = await asyncio.gather(lab.state(19310), lab.state(19320))
                return min(a['blockCount'], b['blockCount']) >= max(before_a['blockCount'], before_b['blockCount'])
            await lab.wait_for(branches_known)
            partition.compose('up', '-d', '--no-deps', 'miner')
            async def joined():
                return (await lab.state(19310))['blockCount'] >= before_a['blockCount'] + before_b['blockCount'] - baseline['blockCount'] + 2
            await lab.wait_for(joined)
            partition.compose('stop', '-t', '2', 'miner')
            final_state = await lab.wait_for(lab.equal_states)
            final_entries = await snapshot(19320)
            if keyed(final_entries) != keyed(await snapshot(19310)):
                raise RuntimeError("B reward snapshots differ between nodes")
            old, new = keyed(before_entries), keyed(final_entries)
            removed = old.keys() - new.keys()
            if not removed or final_state['sink'] == before_b['sink']:
                raise RuntimeError('Expected a sink change and actual UTXO removals')
            async def saw_removal():
                return bool(removed & removed_notifications)
            await lab.wait_for(saw_removal, seconds=15)
            store.reconcile(final_entries)
            after = store.summary()
            for txid, index in removed:
                active = store.db.execute('SELECT active FROM outputs WHERE txid=? AND idx=?', (txid, index)).fetchone()
                if active != (0,):
                    raise RuntimeError('Removed output remained active or history was deleted')
            rows = {(txid, index): amount for txid, index, amount in store.db.execute('SELECT txid,idx,amount FROM outputs WHERE active=1')}
            if rows != new:
                raise RuntimeError('Stored active outputs differ from RPC')
            store.reconcile(final_entries)
            if store.summary() != after:
                raise RuntimeError('Repeated recovery double counted')
            store.close(); store = None
            reopened = json.loads(subprocess.check_output([sys.executable, str(ROOT / 'utxo-store.py'), database], text=True))
            if reopened != after:
                raise RuntimeError('Recovered state did not persist')
            print(json.dumps({'network':'devnet', 'baselineBlocks':baseline['blockCount'],
                              'partitionA':before_a, 'partitionB':before_b, 'converged':final_state,
                              'utxoNotificationCount':notification_count,
                              'removedOutpoints':len(removed), 'removedOutpointsSeenInNotification':len(removed & removed_notifications),
                              'removedAmountSompi':str(sum(old[p] for p in removed)),
                              'beforeRecovery':prior, 'afterRecovery':after,
                              'removedHistoryRetainedInactive':True, 'activeRowsMatchedRpc':True,
                              'repeatedRecoveryIdempotent':True, 'processReopenMatched':True,
                              'databasePublished':False}, indent=2))
    finally:
        if watcher and watcher.returncode is None:
            watcher.terminate()
            await asyncio.wait_for(watcher.wait(), 10)
        if reader:
            await reader
        if store:
            store.close()
        partition.compose('stop', '-t', '2')


if __name__ == '__main__':
    asyncio.run(main())
