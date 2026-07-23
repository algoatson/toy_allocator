mod chunk;
mod error;
mod utils;

use chunk::{
    ChunkHeader, ChunkInfo, ChunkIter, ChunkState, FreeChunk,
    chunk_to_mem, mem_to_chunk
};

use utils::align_up;
use error::AllocError;

const MAGIC: u64 = 0xDEADBEEFDEADBEEF;

pub struct Arena {
    buf: Vec<u8>,
    offset: usize,
    free_list: *mut FreeChunk,
}

impl Arena {
    pub fn new(size: usize) -> Self {
        Self {
            buf: vec![0; align_up(size, 16)],
            offset: 0,
            free_list: std::ptr::null_mut(),
        }
    }

    pub fn chunks(&self) -> impl Iterator<Item=ChunkInfo> {
        ChunkIter {
            arena: self,
            offset: 0,
        }
    }

    fn is_valid_pointer(&self, ptr: *mut u8) -> bool {
        !ptr.is_null() && ptr.is_aligned()
    }

    pub fn dump(&self) {
        for (i, chunk) in self.chunks().enumerate() {
            println!(
                "#{i}: size={} allocated={} state={:?}",
                chunk.requested_size,
                chunk.allocated_size,
                chunk.state
            );
        }
    }

    fn try_alloc(&mut self, size: usize) -> Result<*mut u8, AllocError> {
        // each allocation is padded by ChunkHeader
        // and are 16-byte aligned
        let total = align_up(
            size_of::<ChunkHeader>() + size,
            16
        );

        let mut current = self.free_list;

        while !current.is_null() {
            unsafe {
                let chunk = &mut *current;

                // if chunk.header.size() >= size {
                //     self.remove_free_chunk(current);
                //     return Ok(chunk_to_mem(current as *mut ChunkHeader));
                // }

                current = chunk.next;
            }
        }

        // check in the freelist.
        // for chunk in free_list {
            // if chunk.size >= requested {
                // remove(chunk);
                // return chunk_to_mem(chunk);
            // }
        // }

        // for chunk in self.free_list {

        

        // not enough memory to serve allocation
        if self.offset + total > self.buf.len() {
            return Err(AllocError::OutOfMemory);
        }

        let chunk = unsafe {
            self.buf.as_mut_ptr()
                .add(self.offset)
        };

        #[cfg(debug_assertions)] 
        {
            println!("allocated {:#x} bytes ({:p})", total, chunk)
        }

        unsafe {
            (chunk as *mut ChunkHeader).write(
                ChunkHeader::new(size, total));
        }

        self.offset += total;

        debug_assert!(self.offset <= self.buf.len());

        Ok(chunk_to_mem(chunk as *mut ChunkHeader))

    }

    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        self.try_alloc(size)
            .expect("arena out of memory")
    }

    pub fn free(&mut self, ptr: *mut u8) -> Result<(), AllocError> {
        // validate user-provided pointer. (make sure it's 16-byte aligned and not null)
        if !self.is_valid_pointer(ptr) {
            return Err(AllocError::InvalidPointer);
        }

        // we keep a raw version of the header, and a ref to it
        let raw_header = mem_to_chunk(ptr);
        let header = unsafe { &mut *raw_header };

        // validate magic bytes
        if !header.is_valid() {
            // if the magic-bytes are invalid, it means our chunk
            // is in a corrupted state.
            header.set_state(ChunkState::Corrupted);
            return Err(AllocError::CorruptedChunk);
        }

        // check for double-free's
        match header.chunk_state() {
            ChunkState::Allocated => {
                // chunk is now free
                header.set_state(ChunkState::Free);
                let chunk = raw_header as *mut FreeChunk;

                unsafe {
                    // set the chunk->next to null or to the
                    // head of the list
                    (*chunk).prev = std::ptr::null_mut();
                    (*chunk).next = self.free_list;

                    if !self.free_list.is_null() {
                        (*self.free_list).prev = chunk;
                    }
                }

                self.free_list = chunk;
                Ok(())
            }

            ChunkState::Free => {
                // if the chunk was already free then trigger the
                // double-free error
                Err(AllocError::DoubleFree)
            }

            _ => {
                // anything else means the chunk has been corrupted
                Err(AllocError::CorruptedChunk)
            }
        }
    }
}