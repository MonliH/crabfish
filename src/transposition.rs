use rustc_hash::FxHasher;
use std::{hash::Hasher, ptr};

use crate::score::ScoreTy;

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Flag {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

impl Default for Flag {
    fn default() -> Self {
        Flag::LowerBound
    }
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct CacheItem {
    pub depth: u8,
    pub flag: Flag,
    pub value: ScoreTy,
    pub board_hash: u64,
    pub checksum: u64,
}

impl Default for CacheItem {
    fn default() -> Self {
        Self::new(0, Flag::Exact, 0, 0)
    }
}

impl CacheItem {
    pub fn new(depth: u8, flag: Flag, value: ScoreTy, board_hash: u64) -> Self {
        Self {
            depth,
            flag,
            value,
            board_hash,
            checksum: Self::cache_checksum(depth, flag, value, board_hash),
        }
    }

    fn cache_checksum(depth: u8, flag: Flag, value: ScoreTy, board_hash: u64) -> u64 {
        let mut hasher = FxHasher::default();
        hasher.write_u8(depth);
        hasher.write_u8(flag as u8);
        hasher.write_i16(value);
        hasher.write_u64(board_hash);
        hasher.finish()
    }

    fn checksum_is_valid(&self) -> bool {
        self.checksum == Self::cache_checksum(self.depth, self.flag, self.value, self.board_hash)
    }
}

#[derive(Debug, Clone)]
/// A multithreaded lock free implementation of a transposition table.
///
/// If the checksum of a value is not okay (e.g., if two threads write at the same time),
/// the value is simply discarded on read.
pub struct TTable {
    pub entries: *mut CacheItem,
    pub size: usize,
    pub mask: usize,
}

// SAFTEY: We've accounted for the problems with two simultaneous writers via a checksum.
unsafe impl Send for TTable {}
unsafe impl Sync for TTable {}

impl TTable {
    pub fn new(size: usize) -> Self {
        if size.count_ones() != 1 {
            panic!("Size must be a power of two");
        }
        let mut entries = vec![CacheItem::default(); size];
        entries.shrink_to_fit();
        let entries_ptr = entries.as_mut_ptr();
        std::mem::forget(entries);
        Self {
            size,
            mask: size - 1,
            entries: entries_ptr,
        }
    }

    #[inline]
    pub fn get(&self, hash: u64) -> Option<CacheItem> {
        let entries = ptr::slice_from_raw_parts(self.entries, self.size);
        // SAFTEY: We know the hash `&` the mask is always going to be in bounds.
        // We must clone the item because it might change otherwise.
        let possible_entry: CacheItem =
            unsafe { (&*entries).get((hash as usize) & self.mask).unwrap() }.clone();

        if possible_entry.board_hash == hash && possible_entry.checksum_is_valid() {
            Some(possible_entry)
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&self, item: CacheItem) {
        let entries = ptr::slice_from_raw_parts_mut(self.entries, self.size);

        let possible_entry: &mut CacheItem = unsafe {
            (&mut *entries)
                .get_mut((item.board_hash as usize) & self.mask)
                .unwrap()
        };

        *possible_entry = item;
    }

    #[inline]
    pub fn free(self) {
        // SAFTEY: This operation is safe because we never change the length of the "vector" (i.e,
        // pointer). Thus, we can use the mask (which is len-1) as the size and capacity. The
        // pointer to the data is safe because we forget the data it's pointing to.
        let original_vec = unsafe { Vec::from_raw_parts(self.entries, self.size, self.size) };
        std::mem::drop(original_vec);
    }
}
