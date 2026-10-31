#!/usr/bin/env python3
"""Cut only the P2P relay while both Devnet nodes continue running."""
import asyncio
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).parent
spec = importlib.util.spec_from_file_location('two_node', ROOT / 'two-node.py')
lab = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lab)


def compose(*args):
    return subprocess.run(['docker', 'compose', '-f', str(ROOT / 'two-node.yaml'),
                           '-f', str(ROOT / 'partition.yaml'), '--profile', 'mining', *args],
                          check=True, stdout=subprocess.DEVNULL, timeout=30)


def container_ip(service):
    cid = subprocess.check_output(['docker', 'compose', '-f', str(ROOT / 'two-node.yaml'),
                                   '-f', str(ROOT / 'partition.yaml'), 'ps', '-q', service], text=True).strip()
    meta = json.loads(subprocess.check_output(['docker', 'inspect', cid], text=True))[0]
    return next(iter(meta['NetworkSettings']['Networks'].values()))['IPAddress']


async def peer_count(port):
    return len((await lab.rpc(port, 'getConnectedPeerInfo')).get('peerInfo', []))


async def main():
    try:
        compose('up', '-d', 'node-a', 'relay')
        os.environ['KASPA_LAB_RELAY_IP'] = container_ip('relay')
        compose('up', '-d', 'node-b')
        async def connected():
            counts = await asyncio.gather(peer_count(19310), peer_count(19320))
            return all(n > 0 for n in counts)
        await lab.wait_for(connected)
        baseline = await lab.wait_for(lab.equal_states)
        compose('stop', '-t', '2', 'relay')
        async def disconnected():
            counts = await asyncio.gather(peer_count(19310), peer_count(19320))
            return all(n == 0 for n in counts)
        await lab.wait_for(disconnected)
        # Mine sequentially while separated, keeping the CPU budget to one.
        for service, port in [('miner', 19310), ('miner-b', 19320)]:
            compose('up', '-d', '--no-deps', service)
            async def advanced():
                state = await lab.state(port)
                return state if state['blockCount'] >= baseline['blockCount'] + 5 else None
            await lab.wait_for(advanced)
            compose('stop', '-t', '2', service)
        a, b = await asyncio.gather(lab.state(19310), lab.state(19320))
        if a['sink'] == b['sink']:
            raise RuntimeError('Expected divergent sinks while separated')
        if await peer_count(19310) or await peer_count(19320):
            raise RuntimeError('P2P was not isolated')
        compose('up', '-d', '--no-deps', 'relay')
        if container_ip('relay') != os.environ['KASPA_LAB_RELAY_IP']:
            raise RuntimeError('Relay IP changed')
        await lab.wait_for(connected)
        recovered = await lab.wait_for(lab.equal_states)
        if recovered['blockCount'] <= max(a['blockCount'], b['blockCount']):
            raise RuntimeError('Expected blocks from both branches')
        print(json.dumps({'network': 'devnet', 'baseline': baseline, 'partitionA': a,
                          'partitionB': b, 'zeroPeersWhileSeparated': True,
                          'nodesRemainedResponsive': True, 'reconnected': True,
                          'converged': recovered, 'rewardUtxosMatched': True}, indent=2))
    finally:
        compose('stop', '-t', '2')


if __name__ == '__main__':
    asyncio.run(main())
