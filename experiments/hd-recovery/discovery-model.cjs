// Educational history-based gap policy; not the Kaspa Wallet API's algorithm.
'use strict';
function scanBranch(used, gap, limit) {
  if (!Number.isSafeInteger(gap) || gap <= 0 || !Number.isSafeInteger(limit) || limit <= 0 || limit > 0x80000000) throw Error('Invalid scan bounds');
  if (used.some(i => !Number.isSafeInteger(i) || i < 0 || i >= 0x80000000)) throw Error('Invalid index');
  const seen = new Set(used), found = [];
  let empty = 0, scanned = 0;
  while (scanned < limit && empty < gap) {
    if (seen.has(scanned)) { found.push(scanned); empty = 0; }
    else empty++;
    scanned++;
  }
  return { found, scanned, reason: empty >= gap ? 'gap' : 'bound' };
}
function verifyModel() {
  const fixtures = [
    { name: 'empty', used: [], gap: 20, limit: 50, expected: { found: [], scanned: 20, reason: 'gap' } },
    { name: 'index-19-found', used: [19], gap: 20, limit: 50, expected: { found: [19], scanned: 40, reason: 'gap' } },
    { name: 'index-20-missed', used: [20], gap: 20, limit: 50, expected: { found: [], scanned: 20, reason: 'gap' } },
    { name: 'index-21-missed', used: [21], gap: 20, limit: 50, expected: { found: [], scanned: 20, reason: 'gap' } },
    { name: 'utxo-only-spent-zero', used: [20], gap: 20, limit: 50, expected: { found: [], scanned: 20, reason: 'gap' } },
    { name: 'history-resets-at-spent-zero', used: [0, 20], gap: 20, limit: 50, expected: { found: [0, 20], scanned: 41, reason: 'gap' } },
    { name: 'gap-after-used-zero', used: [0, 21], gap: 20, limit: 50, expected: { found: [0], scanned: 21, reason: 'gap' } },
    { name: 'explicit-bound', used: [21], gap: 23, limit: 22, expected: { found: [21], scanned: 22, reason: 'bound' } },
  ];
  const check = v => { if (!v) throw Error('Gap-model mismatch'); };
  for (const f of fixtures) check(JSON.stringify(scanBranch(f.used, f.gap, f.limit)) === JSON.stringify(f.expected));
  // Independent oracle: enumerate empty windows instead of maintaining a counter.
  let comparisons = 0;
  for (let mask = 0; mask < 256; mask++) for (let gap = 1; gap <= 4; gap++) for (let limit = 1; limit <= 12; limit++) {
    const used = Array.from({ length: 8 }, (_, i) => i).filter(i => mask & (1 << i));
    const ends = Array.from({ length: limit }, (_, i) => i + 1).filter(end => end >= gap &&
      Array.from({ length: gap }, (_, i) => end - 1 - i).every(i => !used.includes(i)));
    const scanned = ends.length ? ends[0] : limit;
    const expected = { found: used.filter(i => i < scanned), scanned, reason: ends.length ? 'gap' : 'bound' };
    check(JSON.stringify(scanBranch(used, gap, limit)) === JSON.stringify(expected));
    comparisons++;
  }
  let rejectedBounds = 0;
  for (const args of [[[], 0, 20], [[], 20, 0], [[-1], 20, 30], [[0x80000000], 20, 30]]) {
    try { scanBranch(...args); } catch { rejectedBounds++; }
  }
  check(rejectedBounds === 4);
  return { fixtures: fixtures.map(f => ({ name: f.name, ...f.expected })), exhaustiveComparisons: comparisons, invalidBoundsRejected: rejectedBounds };
}
module.exports = { scanBranch, verifyModel };
