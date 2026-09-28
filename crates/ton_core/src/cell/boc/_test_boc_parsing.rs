//! Untrusted BoC bytes: hand-written shapes that `CellBuilder` can't produce.

use crate::cell::{BoC, CellType, LevelMask, TonCell, TonHash};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::panic::{AssertUnwindSafe, catch_unwind};

const MAGIC: [u8; 4] = [0xb5, 0xee, 0x9c, 0x72];
const PRUNED_BRANCH: u8 = 1;
const LIBRARY: u8 = 2;
const MERKLE_PROOF: u8 = 3;
const MERKLE_UPDATE: u8 = 4;
/// `vm::CellTraits::max_depth`
const MAX_DEPTH: u16 = 1024;

/// One serialized cell of a hand-written BoC.
#[derive(Clone)]
struct RawCell {
    /// First descriptor byte without the refs count: level mask in the top 3 bits, 0x08 for exotic.
    d1: u8,
    /// Second descriptor byte: `floor(bits / 8) + ceil(bits / 8)`.
    d2: u8,
    /// Data bytes, ending with the completion tag when `d2` is odd.
    data: Vec<u8>,
    /// Indices of referenced cells.
    refs: Vec<usize>,
}

impl RawCell {
    fn ordinary(data: &[u8], refs: Vec<usize>) -> Self {
        Self {
            d1: 0,
            d2: (data.len() * 2) as u8,
            data: data.to_vec(),
            refs,
        }
    }

    fn exotic(mask: u8, data: Vec<u8>, refs: Vec<usize>) -> Self {
        Self {
            d1: level(mask) | 0x08,
            d2: (data.len() * 2) as u8,
            data,
            refs,
        }
    }

    fn with_level(self, mask: u8) -> Self {
        Self {
            d1: self.d1 | level(mask),
            ..self
        }
    }

    /// The same cell placed after `by` other cells.
    fn shifted(self, by: usize) -> Self {
        Self {
            refs: self.refs.iter().map(|pos| pos + by).collect(),
            ..self
        }
    }
}

fn level(mask: u8) -> u8 {
    mask << 5
}

/// A single-root BoC rooted at cell 0: 2-byte refs, 4-byte offsets, no index, no CRC32C.
fn boc(cells: &[RawCell]) -> Vec<u8> {
    let mut body = Vec::new();
    for cell in cells {
        body.push(cell.d1 | cell.refs.len() as u8);
        body.push(cell.d2);
        body.extend_from_slice(&cell.data);
        for &pos in &cell.refs {
            body.extend_from_slice(&(pos as u16).to_be_bytes());
        }
    }
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&[0x02, 0x04]);
    bytes.extend_from_slice(&(cells.len() as u16).to_be_bytes());
    bytes.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]);
    bytes.extend_from_slice(&(body.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&[0x00, 0x00]);
    bytes.extend(body);
    bytes
}

/// Cells `start..start + len` without data, each referencing the next one.
fn chain_cells(start: usize, len: usize) -> Vec<RawCell> {
    let end = start + len;
    (start..end).map(|pos| RawCell::ordinary(&[], if pos + 1 < end { vec![pos + 1] } else { vec![] })).collect()
}

/// A chain of `len` cells, its root is `len - 1` deep.
fn chain(len: usize) -> Vec<u8> {
    boc(&chain_cells(0, len))
}

/// Pruned branch data with `hash_byte + i` filling the i-th stored hash.
fn pruned_with(mask: u8, hash_byte: u8, depths: &[u16]) -> Vec<u8> {
    assert_eq!(depths.len(), mask.count_ones() as usize, "one stored depth per set bit");
    let mut data = vec![PRUNED_BRANCH, mask];
    for i in 0..depths.len() {
        data.extend_from_slice(&[hash_byte + i as u8; 32]);
    }
    for depth in depths {
        data.extend_from_slice(&depth.to_be_bytes());
    }
    data
}

fn pruned(mask: u8, depths: &[u16]) -> Vec<u8> {
    pruned_with(mask, 0x11, depths)
}

/// `len` filler bytes starting with `type_byte`.
fn filled(type_byte: u8, len: usize) -> Vec<u8> {
    let mut data = vec![0x5a; len];
    if let Some(first) = data.first_mut() {
        *first = type_byte;
    }
    data
}

/// A level 1 ordinary cell over a mask 1 pruned branch, as a Merkle proof carries it,
/// with the level 0 hash and depth a Merkle cell stores for it.
fn merkle_child() -> ([RawCell; 2], Vec<u8>, Vec<u8>) {
    let child = [
        RawCell::ordinary(&[], vec![1]).with_level(1),
        RawCell::exotic(1, pruned(1, &[5]), vec![]),
    ];
    let root = parse(&boc(&child)).unwrap();
    let hash = root.hash_for_level(LevelMask::new(0)).unwrap().as_slice().to_vec();
    let depth = root.depth_for_level(LevelMask::new(0)).unwrap();
    assert_eq!(depth, 6);
    (child, hash, depth.to_be_bytes().to_vec())
}

fn merkle_proof(hash: &[u8], depth: &[u8]) -> Vec<u8> {
    let (child, ..) = merkle_child();
    let proof = RawCell::exotic(0, [&[MERKLE_PROOF][..], hash, depth].concat(), vec![1]);
    let cells: Vec<_> = std::iter::once(proof).chain(child.map(|cell| cell.shifted(1))).collect();
    boc(&cells)
}

fn merkle_update(old_hash: &[u8], new_hash: &[u8], old_depth: &[u8], new_depth: &[u8]) -> Vec<u8> {
    let (child, ..) = merkle_child();
    let update =
        RawCell::exotic(0, [&[MERKLE_UPDATE][..], old_hash, new_hash, old_depth, new_depth].concat(), vec![1, 3]);
    let cells: Vec<_> = std::iter::once(update)
        .chain(child.clone().map(|cell| cell.shifted(1)))
        .chain(child.map(|cell| cell.shifted(3)))
        .collect();
    boc(&cells)
}

fn parse(bytes: &[u8]) -> Result<TonCell, TonCoreError> {
    BoC::from_bytes(bytes.to_vec())?.single_root()
}

fn on_stack<T: Send + 'static>(size: usize, job: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(size).spawn(job).unwrap().join().unwrap()
}

#[test]
fn test_boc_rejects_counts_not_backed_by_input() {
    // 2^32 - 1 cells claimed in 23 bytes: allocating for them aborts the process.
    let mut huge_cells = MAGIC.to_vec();
    huge_cells.extend_from_slice(&[0x04, 0x01]);
    huge_cells.extend_from_slice(&u32::MAX.to_be_bytes());
    huge_cells.extend_from_slice(&1u32.to_be_bytes());
    huge_cells.extend_from_slice(&0u32.to_be_bytes());
    huge_cells.push(0xff);
    huge_cells.extend_from_slice(&0u32.to_be_bytes());
    assert!(parse(&huge_cells).is_err());

    // 2^32 - 1 roots.
    let mut huge_roots = MAGIC.to_vec();
    huge_roots.extend_from_slice(&[0x04, 0x01]);
    huge_roots.extend_from_slice(&u32::MAX.to_be_bytes());
    huge_roots.extend_from_slice(&u32::MAX.to_be_bytes());
    huge_roots.extend_from_slice(&0u32.to_be_bytes());
    huge_roots.push(0xff);
    assert!(parse(&huge_roots).is_err());

    // An index and cell data larger than the input, with sizes overflowing u64 when summed.
    let mut huge_index = MAGIC.to_vec();
    huge_index.extend_from_slice(&[0x84, 0x08]);
    huge_index.extend_from_slice(&u32::MAX.to_be_bytes());
    huge_index.extend_from_slice(&1u32.to_be_bytes());
    huge_index.extend_from_slice(&0u32.to_be_bytes());
    huge_index.extend_from_slice(&u64::MAX.to_be_bytes());
    huge_index.extend_from_slice(&0u32.to_be_bytes());
    assert!(parse(&huge_index).is_err());
    huge_index[18..26].copy_from_slice(&0x2_0000_0000u64.to_be_bytes());
    assert!(parse(&huge_index).is_err());
}

#[test]
fn test_boc_rejects_out_of_range_root_and_ref_indices() {
    let bad_root = [
        0xb5, 0xee, 0x9c, 0x72, 0x01, 0x01, 0x01, 0x01, 0x00, 0x02, 0x01, 0x00, 0x00,
    ];
    assert!(parse(&bad_root).is_err());
    let ref_past_last_cell = [
        0xb5, 0xee, 0x9c, 0x72, 0x01, 0x01, 0x01, 0x01, 0x00, 0x03, 0x00, 0x01, 0x00, 0x01,
    ];
    assert!(parse(&ref_past_last_cell).is_err());
    let ref_to_itself = [
        0xb5, 0xee, 0x9c, 0x72, 0x01, 0x01, 0x01, 0x01, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00,
    ];
    assert!(parse(&ref_to_itself).is_err());
}

#[test]
fn test_boc_requires_exact_input_size() {
    assert!(parse(TonCell::EMPTY_BOC).is_ok());
    assert!(parse(&[TonCell::EMPTY_BOC, &[0]].concat()).is_err());
    assert!(parse(&TonCell::EMPTY_BOC[..TonCell::EMPTY_BOC.len() - 1]).is_err());

    // tot_cells_size larger than the cells it holds.
    let mut unused_bytes = boc(&[RawCell::ordinary(&[], vec![])]);
    unused_bytes[13] += 1;
    unused_bytes.push(0);
    assert!(parse(&unused_bytes).is_err());
}

#[test]
fn test_boc_verifies_index() {
    // The shape the TON node writes with an index: 1-byte refs and offsets, one empty cell ending at 2.
    let indexed = [
        0xb5, 0xee, 0x9c, 0x72, 0x81, 0x01, 0x01, 0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00,
    ];
    assert!(parse(&indexed).is_ok());

    let mut wrong_entry = indexed;
    wrong_entry[11] = 0x01;
    assert!(parse(&wrong_entry).is_err());

    // With cache bits, entries hold the end offset shifted left by one.
    let mut cached = indexed;
    cached[4] |= 0x20;
    cached[11] = 0x05;
    assert!(parse(&cached).is_ok());
    cached[11] = 0x02;
    assert!(parse(&cached).is_err());

    let mut cache_bits_without_index = TonCell::EMPTY_BOC.to_vec();
    cache_bits_without_index[4] |= 0x20;
    assert!(parse(&cache_bits_without_index).is_err());
}

#[test]
fn test_boc_verifies_crc32c() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    builder.write_bits([0xde, 0xad], 16)?;
    let crc_boc = BoC::new(builder.build()?).to_bytes(true)?;
    assert!(parse(&crc_boc).is_ok());

    for pos in [crc_boc.len() - 1, crc_boc.len() - 5, 4] {
        let mut corrupt = crc_boc.clone();
        corrupt[pos] ^= 1;
        assert!(parse(&corrupt).is_err(), "byte {pos} flipped");
    }
    Ok(())
}

#[test]
fn test_boc_rejects_invalid_header_sizes() {
    for (pos, value) in [(4, 0x00), (4, 0x05), (5, 0x00), (5, 0x09)] {
        let mut bytes = TonCell::EMPTY_BOC.to_vec();
        bytes[pos] = value;
        assert!(parse(&bytes).is_err(), "byte {pos} = {value:#04x}");
    }
}

#[test]
fn test_boc_rejects_non_canonical_cells() {
    // Odd d2 means the last byte carries a completion tag: 0x00 has none, 0x80 holds no data bits.
    for last_byte in [0x00, 0x80] {
        let cell = RawCell {
            d2: 1,
            ..RawCell::ordinary(&[last_byte], vec![])
        };
        assert!(parse(&boc(&[cell])).is_err(), "last byte {last_byte:#04x}");
    }
    let cell = RawCell {
        d2: 1,
        ..RawCell::ordinary(&[0x40], vec![])
    };
    assert_eq!(parse(&boc(&[cell])).unwrap().data_len_bits(), 1);

    // 5..=7 refs: 7 marks an absent cell.
    for refs in [5u8, 6, 7] {
        let mut bytes = boc(&[RawCell::ordinary(&[], vec![])]);
        let d1 = bytes.len() - 2;
        bytes[d1] = refs;
        bytes.extend(std::iter::repeat_n(0, refs as usize * 2));
        let size = bytes.len() - 16;
        bytes[10..14].copy_from_slice(&(size as u32).to_be_bytes());
        assert!(parse(&bytes).is_err(), "{refs} refs");
    }

    // Level mask that the cell doesn't have.
    assert!(parse(&boc(&[RawCell::ordinary(&[], vec![]).with_level(1)])).is_err());
    assert!(
        parse(&boc(&[
            RawCell::ordinary(&[], vec![1]),
            RawCell::exotic(1, pruned(1, &[5]), vec![])
        ]))
        .is_err()
    );
}

#[test]
fn test_boc_accepts_cells_at_the_depth_bound_and_rejects_deeper_ones() {
    // A small stack: neither parsing nor dropping the tree recurses.
    let (at_bound, deeper) = on_stack(256 * 1024, || (parse(&chain(1025)).map(|_| ()), parse(&chain(1026)).is_err()));
    assert!(at_bound.is_ok());
    assert!(deeper);
    assert!(on_stack(256 * 1024, || parse(&chain(50_000)).is_err()));
}

#[test]
fn test_boc_checks_depth_of_unreachable_cells() {
    let with_unreachable_chain = |len: usize| {
        let mut cells = vec![RawCell::ordinary(&[], vec![1]), RawCell::ordinary(&[], vec![])];
        cells.extend(chain_cells(2, len));
        boc(&cells)
    };
    assert!(parse(&with_unreachable_chain(1025)).is_ok());
    assert!(parse(&with_unreachable_chain(1026)).is_err());
}

#[test]
fn test_boc_depth_follows_the_longest_ref_path() -> anyhow::Result<()> {
    let mut deep_last = vec![RawCell::ordinary(&[], vec![1, 2]), RawCell::ordinary(&[], vec![])];
    deep_last.extend(chain_cells(2, 1025));
    assert!(parse(&boc(&deep_last)).is_err());

    // 4^1024 ref paths: only a linear pass over the cells finishes.
    let ladder: Vec<_> =
        (0..1025).map(|pos| RawCell::ordinary(&[], if pos < 1024 { vec![pos + 1; 4] } else { vec![] })).collect();
    let root = parse(&boc(&ladder))?;
    assert_eq!(root.depth()?, MAX_DEPTH);
    Ok(())
}

#[test]
fn test_boc_accepts_well_formed_exotic_cells() -> anyhow::Result<()> {
    let mut bocs = vec![];
    for mask in 1..=7u8 {
        let depths = vec![MAX_DEPTH; mask.count_ones() as usize];
        bocs.push((format!("pruned mask {mask}"), boc(&[RawCell::exotic(mask, pruned(mask, &depths), vec![])])));
    }
    bocs.push(("library".to_string(), boc(&[RawCell::exotic(0, filled(LIBRARY, 33), vec![])])));
    let (_, hash, depth) = merkle_child();
    bocs.push(("merkle proof".to_string(), merkle_proof(&hash, &depth)));
    bocs.push(("merkle update".to_string(), merkle_update(&hash, &hash, &depth, &depth)));

    for (name, bytes) in bocs {
        let cell = parse(&bytes).map_err(|err| anyhow::anyhow!("{name}: {err}"))?;
        let reparsed = parse(&BoC::new(cell.clone()).to_bytes(true)?)?;
        assert_eq!(reparsed.hash()?, cell.hash()?, "{name}");
    }

    for (cell_type, data) in [
        (CellType::PrunedBranch, pruned(1, &[5])),
        (CellType::LibraryRef, filled(LIBRARY, 33)),
    ] {
        let mut builder = TonCell::builder_extra(cell_type, data.len());
        builder.write_bits(&data, data.len() * 8)?;
        let cell = builder.build()?;
        assert_eq!(parse(&cell.to_boc()?)?.hash()?, cell.hash()?, "{cell_type:?}");
    }
    Ok(())
}

#[test]
fn test_boc_pruned_branch_keeps_stored_hashes_and_depths() -> anyhow::Result<()> {
    // Masks with gaps store one hash and depth per set bit, not per level.
    for (mask, stored_index) in [
        (2u8, [0, 0, 1, 1]),
        (4, [0, 0, 0, 1]),
        (5, [0, 1, 1, 2]),
        (6, [0, 0, 1, 2]),
    ] {
        let depths: Vec<u16> = (0..mask.count_ones() as u16).map(|i| 100 + i).collect();
        let cell = parse(&boc(&[RawCell::exotic(mask, pruned_with(mask, 0x40, &depths), vec![])]))?;
        for (level, &index) in stored_index.iter().enumerate() {
            let level_mask = LevelMask::new(level as u8);
            if index == depths.len() {
                // The pruned branch's own level
                assert_eq!(cell.depth_for_level(level_mask)?, 0, "mask {mask} level {level}");
                continue;
            }
            let stored_hash = TonHash::from_slice_sized(&[0x40 + index as u8; 32]);
            assert_eq!(cell.hash_for_level(level_mask)?, &stored_hash, "mask {mask} level {level}");
            assert_eq!(cell.depth_for_level(level_mask)?, depths[index], "mask {mask} level {level}");
        }
    }
    Ok(())
}

#[test]
fn test_boc_rejects_malformed_exotic_cells() {
    let one = pruned(1, &[5]);
    let mut short = one.clone();
    short.pop();
    let mut long = one.clone();
    long.push(0);
    let over_leaves = |cell: RawCell| {
        let leaves = cell.refs.len();
        let mut cells = vec![cell];
        cells.extend((0..leaves).map(|_| RawCell::ordinary(&[], vec![])));
        boc(&cells)
    };
    let (_, hash, depth) = merkle_child();
    let other_hash = [0x77; 32];
    let other_depth = 7u16.to_be_bytes();

    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "truncated pruned branch",
            vec![
                0xb5, 0xee, 0x9c, 0x72, 0x01, 0x01, 0x01, 0x01, 0x00, 0x03, 0x00, 0x28, 0x02, 0x01,
            ],
        ),
        (
            "state init with truncated pruned code",
            vec![
                0xb5, 0xee, 0x9c, 0x72, 0x01, 0x01, 0x02, 0x01, 0x00, 0x07, 0x00, 0x21, 0x01, 0x24, 0x01, 0x28, 0x02,
                0x01,
            ],
        ),
        ("pruned level 1, type byte only", boc(&[RawCell::exotic(1, vec![PRUNED_BRANCH], vec![])])),
        ("pruned mask 0", boc(&[RawCell::exotic(0, vec![PRUNED_BRANCH, 0], vec![])])),
        ("pruned mask 8", boc(&[RawCell::exotic(0, vec![PRUNED_BRANCH, 8], vec![])])),
        ("pruned level 0 over mask 1", boc(&[RawCell::exotic(0, one.clone(), vec![])])),
        ("pruned level 1 over mask 3", boc(&[RawCell::exotic(1, pruned(3, &[5, 5]), vec![])])),
        ("pruned one byte short", boc(&[RawCell::exotic(1, short, vec![])])),
        ("pruned one byte long", boc(&[RawCell::exotic(1, long, vec![])])),
        ("pruned with a ref", over_leaves(RawCell::exotic(1, one, vec![1]))),
        ("pruned stored depth 1025", boc(&[RawCell::exotic(1, pruned(1, &[1025]), vec![])])),
        ("pruned second stored depth 1025", boc(&[RawCell::exotic(3, pruned(3, &[5, 1025]), vec![])])),
        (
            "ordinary over a pruned depth 1024",
            boc(&[
                RawCell::ordinary(&[], vec![1]).with_level(1),
                RawCell::exotic(1, pruned(1, &[1024]), vec![]),
            ]),
        ),
        (
            "pruned not byte-aligned",
            boc(&[RawCell {
                d2: 71,
                ..RawCell::exotic(1, pruned(1, &[0x80]), vec![])
            }]),
        ),
        ("library, 32 bytes", boc(&[RawCell::exotic(0, filled(LIBRARY, 32), vec![])])),
        ("library, 34 bytes", boc(&[RawCell::exotic(0, filled(LIBRARY, 34), vec![])])),
        ("library with a ref", over_leaves(RawCell::exotic(0, filled(LIBRARY, 33), vec![1]))),
        ("merkle proof, 34 bytes", over_leaves(RawCell::exotic(0, filled(MERKLE_PROOF, 34), vec![1]))),
        ("merkle proof, 36 bytes", over_leaves(RawCell::exotic(0, filled(MERKLE_PROOF, 36), vec![1]))),
        ("merkle proof without a ref", boc(&[RawCell::exotic(0, filled(MERKLE_PROOF, 35), vec![])])),
        ("merkle proof with two refs", over_leaves(RawCell::exotic(0, filled(MERKLE_PROOF, 35), vec![1, 2]))),
        ("merkle proof with a wrong hash", merkle_proof(&other_hash, &depth)),
        ("merkle proof with a wrong depth", merkle_proof(&hash, &other_depth)),
        ("merkle update, 68 bytes", over_leaves(RawCell::exotic(0, filled(MERKLE_UPDATE, 68), vec![1, 2]))),
        ("merkle update, 70 bytes", over_leaves(RawCell::exotic(0, filled(MERKLE_UPDATE, 70), vec![1, 2]))),
        ("merkle update with one ref", over_leaves(RawCell::exotic(0, filled(MERKLE_UPDATE, 69), vec![1]))),
        ("merkle update with three refs", over_leaves(RawCell::exotic(0, filled(MERKLE_UPDATE, 69), vec![1, 2, 3]))),
        ("merkle update with a wrong new hash", merkle_update(&hash, &other_hash, &depth, &depth)),
        ("merkle update with a wrong old depth", merkle_update(&hash, &hash, &other_depth, &depth)),
        ("exotic without a type byte", boc(&[RawCell::exotic(0, vec![], vec![])])),
        ("exotic type byte 0", boc(&[RawCell::exotic(0, filled(0, 33), vec![])])),
        ("exotic type byte 5", boc(&[RawCell::exotic(0, filled(5, 33), vec![])])),
    ];
    for (name, bytes) in cases {
        let result = catch_unwind(|| parse(&bytes).and_then(|cell| cell.hash().map(|_| ())));
        assert!(matches!(result, Ok(Err(_))), "{name}");
    }
}

#[test]
fn test_boc_drops_deep_trees_without_recursion() -> anyhow::Result<()> {
    let tree = on_stack(64 << 20, || -> anyhow::Result<TonCell> {
        let mut cell = TonCell::empty().to_owned();
        for _ in 0..100_000 {
            let mut builder = TonCell::builder();
            builder.write_ref(cell)?;
            cell = builder.build()?;
        }
        Ok(cell)
    })?;
    on_stack(256 * 1024, move || drop(tree));
    Ok(())
}

/// Deterministic xorshift64, so a failure reproduces.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// Parses and uses every accepted cell tree: none of it may panic.
fn parse_and_use(bytes: &[u8]) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if let Ok(boc) = BoC::from_bytes(bytes.to_vec()) {
            let mut index = 0;
            while let Some(root) = boc.get_root(index) {
                if root.hash().is_ok() {
                    let _ = (root.depth(), root.to_boc(), root.deep_copy(), format!("{root}"));
                }
                index += 1;
            }
            let _ = boc.to_bytes(true);
        }
    }));
    assert!(result.is_ok(), "panicked on {}", hex::encode(bytes));
}

#[test]
fn test_boc_mutated_bytes_never_panic() -> anyhow::Result<()> {
    let (_, hash, depth) = merkle_child();
    let mut seeds = vec![
        TonCell::EMPTY_BOC.to_vec(),
        chain(3),
        boc(&[RawCell::exotic(5, pruned(5, &[1, 2]), vec![])]),
        boc(&[RawCell::exotic(0, filled(LIBRARY, 33), vec![])]),
        merkle_proof(&hash, &depth),
        merkle_update(&hash, &hash, &depth, &depth),
        [
            0xb5, 0xee, 0x9c, 0x72, 0x81, 0x01, 0x01, 0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00,
        ]
        .to_vec(),
    ];
    let mut builder = TonCell::builder();
    builder.write_bits([0xf0], 5)?;
    builder.write_ref(TonCell::empty().to_owned())?;
    seeds.push(BoC::new(builder.build()?).to_bytes(true)?);

    let mut rng = Rng(0x5eed_b0c5);
    for _ in 0..20_000 {
        let mut bytes = seeds[rng.below(seeds.len())].clone();
        for _ in 0..1 + rng.below(4) {
            let pos = rng.below(bytes.len());
            match rng.below(4) {
                0 => bytes[pos] ^= 1 << rng.below(8),
                1 => bytes[pos] = rng.next() as u8,
                2 => bytes.truncate(pos.max(4)),
                _ => bytes.insert(pos, rng.next() as u8),
            }
        }
        parse_and_use(&bytes);
    }
    for _ in 0..20_000 {
        let mut bytes = MAGIC.to_vec();
        bytes.extend((0..rng.below(64)).map(|_| rng.next() as u8));
        parse_and_use(&bytes);
    }
    for _ in 0..20_000 {
        let count = 1 + rng.below(4);
        let cells: Vec<_> = (0..count).map(|pos| random_cell(&mut rng, pos, count)).collect();
        parse_and_use(&boc(&cells));
    }
    Ok(())
}

/// Cell `pos` of `count` with a valid header but exotic-heavy random content:
/// pruned-branch masks and stored depths near the bounds, refs to later cells.
fn random_cell(rng: &mut Rng, pos: usize, count: usize) -> RawCell {
    let mask = [0, 1, 3, 7, rng.below(8) as u8][rng.below(5)];
    let len = [0, 1, 2, 3, 33, 35, 36, 69, 70, rng.below(128)][rng.below(10)];
    let mut data: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
    if let Some(type_byte) = data.first_mut() {
        *type_byte = rng.below(6) as u8;
    }
    if let Some(mask_byte) = data.get_mut(1) {
        *mask_byte = if rng.below(4) == 0 { rng.next() as u8 } else { mask };
    }
    let levels = data.get(1).map_or(0, |mask| mask.count_ones() as usize);
    for level in 0..levels {
        let start = 2 + levels * 32 + level * 2;
        if let Some(slot) = data.get_mut(start..start + 2) {
            slot.copy_from_slice(&[0u16, 5, 1024, 1025, u16::MAX][rng.below(5)].to_be_bytes());
        }
    }
    let later = count - pos - 1;
    let refs = (0..rng.below(5)).filter(|_| later > 0).map(|_| pos + 1 + rng.below(later)).collect();
    match rng.below(4) {
        0 => RawCell::ordinary(&data, refs).with_level(mask),
        _ => RawCell::exotic(mask, data, refs),
    }
}
