use super::Arena;

const MAGIC: u64 = 0xDEADBEEFDEADBEEF;

const STATE_BITS: usize = 2;
const STATE_MASK: usize = (1 << STATE_BITS) - 1;

// 00 = invalid
// 01 = allocated
// 10 = free
// 11 = corrupted/reserved
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    None      = 0b00,
    Allocated = 0b01,
    Free      = 0b10,
    Corrupted = 0b11,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkFlags {
    HasGuard = 1 << 3,
    Zeroes   = 1 << 4,
}

#[repr(C)]
pub(crate) struct ChunkHeader {
    magic: u64,
    size_flags: usize,
    requested_size: usize
}

impl ChunkHeader {
    // pub fn new(size: usize, allocated_size: usize) -> Self {
    //     Self {
    //         magic: MAGIC,
    //         size_flags: allocated_size | ChunkState::Allocated as usize,
    //         requested_size: size,
    //     }
    // }

    fn calculate_magic(
        secret: u64,
        addr: usize,
        requested_size: usize,
        allocated_size: usize,
    ) -> u64 {
        let mut magic = secret;

        magic ^= addr as u64;
        magic ^= requested_size as u64;
        magic ^= allocated_size as u64;

        magic.rotate_left(13)
    }

    pub fn new(
        secret: u64,
        addr: usize,
        size: usize,
        allocated_size: usize
    ) -> Self {
        let magic = Self::calculate_magic(
            secret,
            addr,
            size,
            allocated_size
        );

        Self {
            magic,
            size_flags: allocated_size | ChunkState::Allocated as usize,
            requested_size: size,
        }
    }

    pub(crate) fn is_valid(&self, secret: u64, addr: usize) -> bool {
        let expected = Self::calculate_magic(
            secret, 
            addr, 
            self.requested_size(), 
            self.allocated_size()
        );

        self.magic == expected
    }
    
    pub(crate) fn requested_size(&self) -> usize {
        self.requested_size
    }

    pub(crate) fn allocated_size(&self) -> usize {
        self.size_flags & !STATE_MASK
    }

    pub(crate) fn flags(&self) -> usize {
        self.size_flags & STATE_MASK
    }

    pub(crate) fn chunk_state(&self) -> ChunkState {
        match self.size_flags & 0b1111 {
            0b00 => ChunkState::None,
            0b01 => ChunkState::Allocated,
            0b10 => ChunkState::Free,
            0b11 => ChunkState::Corrupted,
            _ => panic!("invalid state"),
        }
    }

    pub(crate) fn set_state(&mut self, state: ChunkState) {
        self.size_flags &= !STATE_MASK;
        self.size_flags |= state as usize;
    }
}

pub fn chunk_to_mem(ptr: *mut ChunkHeader) -> *mut u8 {
    unsafe { ptr.add(1) as *mut u8 }
}

pub fn mem_to_chunk(ptr: *mut u8) -> *mut ChunkHeader {
    unsafe { ptr.sub(size_of::<ChunkHeader>()) as *mut ChunkHeader }
}

pub struct ChunkInfo {
    pub requested_size: usize,
    pub allocated_size: usize,
    pub state: ChunkState,
    pub magic: u64,
}

pub(crate) struct ChunkIter<'a> {
    pub arena: &'a Arena,
    pub offset: usize,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = ChunkInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.arena.offset {
            return None
        }

        let chunk_ptr = unsafe {
            self.arena.buf
                .as_ptr()
                .add(self.offset)
                as *const ChunkHeader
        };

        let header = unsafe {
            &*chunk_ptr
        };

        self.offset += header.allocated_size();

        Some(ChunkInfo {
            requested_size: header.requested_size(),
            allocated_size: header.allocated_size(),
            state: header.chunk_state(),
            magic: header.magic,
        })
    }
}

// free_list
    // |
    // v
// +--------+
// | chunk  |
// | next --+----+
// +--------+    |
            //  v
        //   +--------+
        //   | chunk  |
        //   | next   |
        //   +--------+

#[repr(C)]
pub(crate) struct FreeChunk {
    pub(crate) header: ChunkHeader,
    pub(crate) next: *mut FreeChunk,
    pub(crate) prev: *mut FreeChunk,
}