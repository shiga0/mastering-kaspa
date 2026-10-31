#!/usr/bin/env python3
"""Read-only smoke test for the book's localhost Devnet, not a sync/E2E test."""
import asyncio
import json

import websockets


async def probe():
    results = {}
    async with websockets.connect("ws://127.0.0.1:18310", open_timeout=10) as ws:
        for request_id, method in enumerate(
            ("getInfo", "getCurrentNetwork", "getBlockDagInfo"), start=1
        ):
            await ws.send(json.dumps({"id": request_id, "method": method, "params": {}}))
            reply = json.loads(await asyncio.wait_for(ws.recv(), timeout=10))
            if reply.get("id") != request_id or reply.get("method") != method:
                raise RuntimeError(f"Unexpected RPC reply: {reply}")
            if reply.get("error") or not isinstance(reply.get("params"), dict):
                raise RuntimeError(f"RPC failed: {reply}")
            results[method] = reply["params"]
    if results["getCurrentNetwork"].get("network") != "devnet":
        raise RuntimeError("Expected Devnet")
    if results["getBlockDagInfo"].get("network") != "devnet":
        raise RuntimeError("DAG network mismatch")
    if results["getInfo"].get("serverVersion") != "1.0.1":
        raise RuntimeError("Unexpected server version")
    if results["getInfo"].get("isUtxoIndexed") is not True:
        raise RuntimeError("UTXO index isn't enabled")
    # isSynced is reported as observed; mining can change it.
    print(json.dumps(results, indent=2, sort_keys=True))


if __name__ == "__main__":
    asyncio.run(probe())
