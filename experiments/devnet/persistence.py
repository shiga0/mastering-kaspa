#!/usr/bin/env python3
"""Snapshot mined DAG/UTXO state; compare after stopping miner and restarting node."""
import argparse
import asyncio
import json
from pathlib import Path

import websockets

ADDRESS = "kaspadev:qpumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtese6y7tc99"


async def snapshot():
    async with websockets.connect("ws://127.0.0.1:18310", open_timeout=10) as ws:
        async def rpc(method, params):
            await ws.send(json.dumps({"id": 1, "method": method, "params": params}))
            response = json.loads(await asyncio.wait_for(ws.recv(), 15))
            if response.get("id") != 1 or response.get("method") != method or response.get("error"):
                raise RuntimeError(response)
            return response["params"]
        network = await rpc("getCurrentNetwork", {})
        if network.get("network") != "devnet":
            raise RuntimeError("Expected Devnet")
        dag = await rpc("getBlockDagInfo", {})
        if dag["blockCount"] <= 0:
            raise RuntimeError("Mine blocks before taking a persistence snapshot")
        utxos = await rpc("getUtxosByAddresses", {"addresses": [ADDRESS]})
        entries = utxos.get("entries", [])
        if not entries:
            raise RuntimeError("No reward UTXOs observed")
        # The miner must be stopped, so this state should stay constant across restart.
        return {
            "network": network["network"],
            "blockCount": dag["blockCount"],
            "virtualDaaScore": dag["virtualDaaScore"],
            "tipHashes": sorted(dag["tipHashes"]),
            "utxos": sorted(entries, key=lambda x: json.dumps(x, sort_keys=True)),
        }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compare", type=Path)
    args = parser.parse_args()
    state = asyncio.run(snapshot())
    if args.compare and state != json.loads(args.compare.read_text()):
        raise SystemExit("FAIL: DAG or UTXO state changed across restart")
    print(json.dumps(state, indent=2, sort_keys=True))
