#!/usr/bin/env python3
"""Read-only prerequisites: point movement alone never proves pruning/IBD."""
import argparse
import asyncio
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).parent
spec = importlib.util.spec_from_file_location('lab', ROOT / 'two-node.py')
lab = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lab)
GENESIS = '4cb48d0b2073b802360145a15ad1abdc01d89b5c2fe4722630ab9b5fe9dfc4f2'


async def inspect():
    network = await lab.rpc(18310, 'getCurrentNetwork')
    info = await lab.rpc(18310, 'getInfo')
    if network.get('network') != 'devnet' or info.get('serverVersion') != '1.0.1':
        raise RuntimeError('Expected Devnet v1.0.1')
    dag = await lab.rpc(18310, 'getBlockDagInfo')
    point = dag['pruningPointHash']
    block = await lab.rpc(18310, 'getBlock', {'hash': dag['sink'], 'includeTransactions': False})
    sink_blue_score = block['block']['header']['blueScore']
    return {'network':'devnet', 'serverVersion':info['serverVersion'],
            'referenceSourceCommit':'fcd9c28f9b211b9872a0c90a8c94f79466ffb063',
            'sourceDeclaredPruningDepth':185798, 'sourceDeclaredFinalityDepth':86400,
            'sourceCrescendoActivation':'never', 'virtualDaaScore':dag['virtualDaaScore'],
            'blockCount':dag['blockCount'], 'sinkBlueScore':sink_blue_score, 'pruningPointHash':point,
            'pointAdvancedFromGenesis':point != GENESIS,
            'historyRemovalVerified':False, 'pruningProofImportVerified':False,
            'status':'point-advanced-needs-history-and-IBD-checks' if point != GENESIS else 'not-exercised'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--require-advanced', action='store_true')
    args = parser.parse_args()
    result = asyncio.run(inspect())
    print(json.dumps(result, indent=2))
    if args.require_advanced and not result['pointAdvancedFromGenesis']:
        raise SystemExit(2)
