#!/usr/bin/env python3
"""Durable current-UTXO observations, not an accounting/payment ledger."""
import json
import sqlite3
import sys


class Store:
    def __init__(self, filename):
        self.db = sqlite3.connect(filename, timeout=5)
        self.db.execute('''CREATE TABLE IF NOT EXISTS outputs (
            txid TEXT NOT NULL, idx INTEGER NOT NULL, amount INTEGER NOT NULL CHECK(amount >= 0),
            active INTEGER NOT NULL CHECK(active IN (0,1)), PRIMARY KEY(txid, idx))''')

    def _put(self, entry):
        outpoint = entry['outpoint']
        amount = int(entry['utxoEntry']['amount'])
        if amount < 0 or amount > 2**63 - 1:
            raise ValueError('Amount outside storage range')
        previous = self.db.execute('SELECT amount FROM outputs WHERE txid=? AND idx=?',
                                   (outpoint['transactionId'], outpoint['index'])).fetchone()
        if previous and previous[0] != amount:
            raise ValueError('Inconsistent amount for existing outpoint')
        self.db.execute('''INSERT INTO outputs VALUES (?, ?, ?, 1)
            ON CONFLICT(txid,idx) DO UPDATE SET active=1''',
                        (outpoint['transactionId'], outpoint['index'], amount))

    def apply(self, added, removed):
        with self.db:
            for entry in removed:
                point = entry['outpoint']
                self.db.execute('UPDATE outputs SET active=0 WHERE txid=? AND idx=?',
                                (point['transactionId'], point['index']))
            for entry in added:
                self._put(entry)

    def reconcile(self, entries):
        # Only call with a complete, successful snapshot of the monitored scope.
        with self.db:
            self.db.execute('UPDATE outputs SET active=0')
            for entry in entries:
                self._put(entry)

    def summary(self):
        count, amount = self.db.execute('SELECT count(*), coalesce(sum(amount),0) FROM outputs WHERE active=1').fetchone()
        return {'activeOutpoints': count, 'activeAmountSompi': str(amount)}

    def close(self):
        self.db.close()


class LatestSnapshot:
    """Single-process request fence; never apply a notification delta here."""
    def __init__(self, store):
        self.store = store
        self.requested = 0
        self.committed = 0

    def begin(self):
        self.requested += 1
        return self.requested

    def commit(self, generation, entries):
        if generation != self.requested or generation <= self.committed:
            return False
        self.store.reconcile(entries)
        self.committed = generation
        return True


if __name__ == '__main__':
    store = Store(sys.argv[1])
    try:
        print(json.dumps(store.summary()))
    finally:
        store.close()
