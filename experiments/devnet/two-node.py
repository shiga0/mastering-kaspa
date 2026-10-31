#!/usr/bin/env python3
"""Devnet-only two-node propagation and restart catch-up experiment."""
import asyncio
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

import websockets

COMPOSE = str(Path(__file__).with_name('two-node.yaml'))
ADDRESS = 'kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99'


def compose(*args):
    subprocess.run(['docker', 'compose', '-f', COMPOSE, '--profile', 'mining', *args],
                   check=True, stdout=subprocess.DEVNULL, timeout=30)


async def rpc(port, method, params=None):
    async with websockets.connect(f'ws://127.0.0.1:{port}', open_timeout=3) as ws:
        await ws.send(json.dumps({'id': 1, 'method': method, 'params': params or {}}))
        reply = json.loads(await asyncio.wait_for(ws.recv(), 5))
        if reply.get('id') != 1 or reply.get('method') != method or reply.get('error'):
            raise RuntimeError('Unexpected RPC response')
        return reply['params']


async def wait_for(check, seconds=90):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        try:
            result = await check()
            if result:
                return result
        except (OSError, EOFError, asyncio.TimeoutError, websockets.exceptions.WebSocketException):
            pass  # RPC endpoint may still be starting; protocol errors are not retried.
        await asyncio.sleep(.25)
    raise RuntimeError('Condition deadline exceeded')


async def state(port):
    dag = await rpc(port, 'getBlockDagInfo')
    if dag.get('network') != 'devnet':
        raise RuntimeError('Expected Devnet')
    info = await rpc(port, 'getInfo')
    if info.get('serverVersion') != '1.0.1' or info.get('isUtxoIndexed') is not True:
        raise RuntimeError('Unexpected node version or index configuration')
    entries = (await rpc(port, 'getUtxosByAddresses', {'addresses': [ADDRESS]}))['entries']
    encoded = json.dumps(sorted(entries, key=lambda x: json.dumps(x, sort_keys=True)), sort_keys=True).encode()
    return {'blockCount': dag['blockCount'], 'virtualDaaScore': dag['virtualDaaScore'],
            'sink': dag['sink'], 'tipHashes': sorted(dag['tipHashes']),
            'rewardUtxoCount': len(entries), 'rewardUtxoSha256': hashlib.sha256(encoded).hexdigest()}


async def equal_states():
    a, b = await asyncio.gather(state(19310), state(19320))
    return a if a == b and a['blockCount'] > 0 and a['rewardUtxoCount'] > 0 else None


async def main():
    try:
        compose('up', '-d', 'node-a')
        container_id = subprocess.check_output(['docker', 'compose', '-f', COMPOSE, 'ps', '-q', 'node-a'], text=True).strip()
        metadata = json.loads(subprocess.check_output(['docker', 'inspect', container_id], text=True))[0]
        os.environ['KASPA_LAB_NODE_A_IP'] = next(iter(metadata['NetworkSettings']['Networks'].values()))['IPAddress']
        compose('up', '-d', 'node-b')
        async def ready():
            return await asyncio.gather(rpc(19310, 'getInfo'), rpc(19320, 'getInfo'))
        await wait_for(ready)
        async def connected():
            a, b = await asyncio.gather(rpc(19310, 'getConnectedPeerInfo'), rpc(19320, 'getConnectedPeerInfo'))
            return len(a.get('peerInfo', [])) > 0 and len(b.get('peerInfo', [])) > 0
        await wait_for(connected)
        baseline = await state(19310)
        compose('up', '-d', 'miner')
        async def advanced():
            a = await state(19310)
            return a if a['blockCount'] >= baseline['blockCount'] + 5 else None
        await wait_for(advanced)
        compose('stop', '-t', '2', 'miner')
        propagated = await wait_for(equal_states)
        compose('stop', 'node-b')
        compose('up', '-d', 'miner')
        async def offline_advanced():
            a = await state(19310)
            return a if a['blockCount'] >= propagated['blockCount'] + 5 else None
        await wait_for(offline_advanced)
        compose('stop', '-t', '2', 'miner')
        offline = await state(19310)
        compose('up', '-d', 'node-b')
        recovered = await wait_for(equal_states)
        if recovered != offline:
            raise RuntimeError('State changed after miner stopped')
        print(json.dumps({'network': 'devnet', 'peerConnectionObservedOnBoth': True,
                          'initialPropagation': propagated, 'aWhileBStopped': offline,
                          'afterBRestart': recovered, 'stateMatchedAfterRestart': True}, indent=2))
    finally:
        compose('stop', '-t', '2')


if __name__ == '__main__':
    asyncio.run(main())
