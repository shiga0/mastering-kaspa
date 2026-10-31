import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('utxo_store', Path(__file__).with_name('utxo-store.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def entry(txid, amount):
    return {'outpoint': {'transactionId': txid, 'index': 0}, 'utxoEntry': {'amount': amount}}


class StoreTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.path = str(Path(self.directory.name) / 'test.sqlite')
        self.store = module.Store(self.path)

    def tearDown(self):
        self.store.close()
        self.directory.cleanup()

    def test_duplicate_add_and_remove_are_idempotent(self):
        value = entry('one', 100)
        self.store.apply([value, value], [])
        self.assertEqual(self.store.summary(), {'activeOutpoints': 1, 'activeAmountSompi': '100'})
        self.store.apply([], [value, value])
        self.assertEqual(self.store.summary()['activeOutpoints'], 0)

    def test_conflicting_amount_rolls_back_entire_batch(self):
        self.store.apply([entry('one', 100)], [])
        before = self.store.summary()
        with self.assertRaises(ValueError):
            self.store.apply([entry('two', 20), entry('one', 101)], [entry('one', 100)])
        self.assertEqual(self.store.summary(), before)

    def test_snapshot_deactivates_missing_and_preserves_history(self):
        self.store.reconcile([entry('one', 100), entry('two', 200)])
        self.store.reconcile([entry('two', 200)])
        self.assertEqual(self.store.summary(), {'activeOutpoints': 1, 'activeAmountSompi': '200'})
        self.store.reconcile([])
        self.assertEqual(self.store.summary()['activeOutpoints'], 0)

    def test_snapshot_failure_rolls_back_after_deactivation(self):
        self.store.reconcile([entry('one', 100)])
        with self.assertRaises(ValueError):
            self.store.reconcile([entry('two', 200), entry('one', 101)])
        self.store.close()
        self.store = module.Store(self.path)
        self.assertEqual(self.store.summary(), {'activeOutpoints': 1, 'activeAmountSompi': '100'})


class SnapshotOrderTests(unittest.TestCase):
    setUp = StoreTests.setUp
    tearDown = StoreTests.tearDown
    def test_late_old_snapshot_cannot_restore_removed_output(self):
        fence = module.LatestSnapshot(self.store)
        old = fence.begin()
        new = fence.begin()
        self.assertTrue(fence.commit(new, [entry('new', 200)]))
        self.assertFalse(fence.commit(old, [entry('old', 100)]))
        self.assertEqual(self.store.summary(), {'activeOutpoints': 1, 'activeAmountSompi': '200'})

    def test_pending_new_request_invalidates_older_result(self):
        fence = module.LatestSnapshot(self.store)
        old = fence.begin()
        fence.begin()
        self.assertFalse(fence.commit(old, [entry('old', 100)]))
        self.assertEqual(self.store.summary()['activeOutpoints'], 0)

    def test_failed_snapshot_does_not_advance_committed_generation(self):
        self.store.reconcile([entry('one', 100)])
        fence = module.LatestSnapshot(self.store)
        generation = fence.begin()
        with self.assertRaises(ValueError):
            fence.commit(generation, [entry('one', 101)])
        self.assertEqual(fence.committed, 0)
        self.assertTrue(fence.commit(generation, [entry('one', 100)]))
        self.assertFalse(fence.commit(generation, []))


if __name__ == '__main__':
    unittest.main()
