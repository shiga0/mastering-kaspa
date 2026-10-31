#!/usr/bin/env python3
"""Notifications are hints; fenced snapshots repair live and reconnect gaps."""
import asyncio
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT = Path(__file__).parent
spec = importlib.util.spec_from_file_location('notifications', ROOT / 'notifications.py')
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


async def start_watcher():
    process = await asyncio.create_subprocess_exec('node', str(ROOT / 'watch-once.cjs'), '--hints',
                                                  stdout=asyncio.subprocess.PIPE,
                                                  stderr=asyncio.subprocess.DEVNULL)
    try:
        ready = json.loads(await asyncio.wait_for(process.stdout.readline(), 15))
        if not ready.get('ready'):
            raise RuntimeError('Subscription failed')
        return process
    except BaseException:
        if process.returncode is None:
            process.terminate()
        await process.wait()
        raise


async def stop_watcher(process, reader):
    if process and process.returncode is None:
        process.terminate()
        await asyncio.wait_for(process.wait(), 10)
    if reader:
        await reader


def active_rows(store):
    return list(store.db.execute('SELECT txid,idx,amount FROM outputs WHERE active=1 ORDER BY txid,idx'))


def rpc_rows(entries):
    return sorted((e['outpoint']['transactionId'], e['outpoint']['index'], int(e['utxoEntry']['amount'])) for e in entries)


async def main():
    watcher = reader = worker = None
    store = None
    try:
        helper.compose('up', '-d', 'node')
        async def ready():
            return {'entries': await helper.snapshot()}
        await helper.lab.wait_for(ready)
        with tempfile.TemporaryDirectory(prefix='kaspa-live-observations-') as directory:
            store = helper.storage.Store(str(Path(directory) / 'observations.sqlite'))
            fence = helper.storage.LatestSnapshot(store)
            hints = 0
            refreshes = 0
            dirty = asyncio.Event()
            running = True

            async def consume(process):
                nonlocal hints
                while line := await process.stdout.readline():
                    if json.loads(line).get('changed'):
                        hints += 1
                        dirty.set()

            async def refresh():
                nonlocal refreshes
                generation = fence.begin()
                entries = await helper.snapshot()
                if fence.commit(generation, entries):
                    refreshes += 1

            async def reconcile_loop():
                while running:
                    try:
                        await asyncio.wait_for(dirty.wait(), 1)
                    except asyncio.TimeoutError:
                        pass  # Periodic snapshots also repair silently lost notifications.
                    dirty.clear()  # A hint received during the query sets it again.
                    await refresh()

            # Subscribe before the initial snapshot, then hold an older query result.
            watcher = await start_watcher()
            reader = asyncio.create_task(consume(watcher))
            old_generation = fence.begin()
            old_entries = await helper.snapshot()
            worker = asyncio.create_task(reconcile_loop())
            dirty.set()
            before = await helper.lab.rpc(18310, 'getBlockDagInfo')
            helper.compose('up', '-d', 'miner')
            async def live_progress():
                dag = await helper.lab.rpc(18310, 'getBlockDagInfo')
                return hints > 0 and refreshes > 1 and dag['blockCount'] >= before['blockCount'] + 5
            await helper.lab.wait_for(live_progress)
            helper.compose('stop', '-t', '2', 'miner')
            await refresh()
            stale_rejected = not fence.commit(old_generation, old_entries)
            if not stale_rejected:
                raise RuntimeError('Old response was applied')
            live_hints = hints
            live_refreshes = refreshes

            await stop_watcher(watcher, reader)
            watcher = reader = None
            running = False
            dirty.set()
            await worker
            worker = None
            frozen_rows = active_rows(store)
            gap_start = await helper.lab.rpc(18310, 'getBlockDagInfo')
            helper.compose('up', '-d', 'miner')
            async def gap_progress():
                dag = await helper.lab.rpc(18310, 'getBlockDagInfo')
                return dag['blockCount'] >= gap_start['blockCount'] + 5
            await helper.lab.wait_for(gap_progress)
            helper.compose('stop', '-t', '2', 'miner')
            final_entries = await helper.snapshot()
            if frozen_rows == rpc_rows(final_entries):
                raise RuntimeError('Expected a missed-update gap')
            watcher = await start_watcher()  # Resubscribe even if no new notification follows.
            reader = asyncio.create_task(consume(watcher))
            await refresh()  # Explicit catch-up on reconnect, not waiting for a notification.
            if active_rows(store) != rpc_rows(final_entries):
                raise RuntimeError('Reconnect snapshot did not repair state')
            print(json.dumps({'network':'devnet', 'liveNotificationHints':live_hints,
                              'liveSnapshotCommits':live_refreshes, 'lateOldSnapshotRejected':stale_rejected,
                              'gapObservedWhileDisconnected':True, 'resubscribed':True,
                              'recoveredWithoutWaitingForNewHint':True, 'outpointsAndAmountsMatched':True,
                              'reconciled':store.summary(), 'databasePublished':False}, indent=2))
    finally:
        if worker:
            worker.cancel()
            await asyncio.gather(worker, return_exceptions=True)
        await stop_watcher(watcher, reader)
        if store:
            store.close()
        helper.compose('stop', '-t', '2')


if __name__ == '__main__':
    asyncio.run(main())
