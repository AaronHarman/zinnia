use std::mem;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Chunk {A, B, None}

// A buffer that stores values in two chunks, and returns a full chunk at a time without copying

pub struct ChunkBuffer<T: Clone> {
    chunk_a : Vec<T>,
    chunk_b : Vec<T>,
    chunk_size : usize,
    current_chunk : Chunk,
    current_pos : usize,
    filled_chunk : Chunk,
}
impl<T: Clone> ChunkBuffer<T> {
    // create a new ChunkBuffer with a given chunk size
    pub fn new(size : usize) -> ChunkBuffer<T> {
        return ChunkBuffer {
            chunk_a : Vec::with_capacity(size),
            chunk_b : Vec::with_capacity(size),
            chunk_size : size,
            current_chunk : Chunk::A,
            current_pos : 0,
            filled_chunk : Chunk::None,
        };
    }

    // check to see if there is a full Vec within the ChunkBuffer
    pub fn full(&self) -> bool {
        return self.filled_chunk != Chunk::None;
    }

    // returns one full chunk from the buffer, replacing it with an empty one
    pub fn pop(&mut self) -> Option<Vec<T>> {
        match self.filled_chunk {
            Chunk::A => {
                return Some(mem::replace(&mut self.chunk_a, Vec::with_capacity(self.chunk_size)));
            },
            Chunk::B => {
                return Some(mem::replace(&mut self.chunk_b, Vec::with_capacity(self.chunk_size)));
            },
            Chunk::None => {
                return None;
            }
        }
    }

    // pushes new values into the buffer, overflowing into the next buffer if necessary
    pub fn push_slice(&mut self, slice : &[T]) {
        let remaining = self.chunk_size-self.current_pos;
        // there's enough space to just toss it in there
        if slice.len() <= remaining {
            match self.current_chunk {
                Chunk::A => {self.chunk_a.extend_from_slice(slice)},
                Chunk::B => {self.chunk_b.extend_from_slice(slice)},
                Chunk::None => {}// if this happens something has gone terribly wrong
            }
            // increment how much is filled in the current chunk
            self.current_pos += slice.len();
            // if we just filled this chunk, make sure to move on
            if self.current_pos == self.chunk_size {
                self.current_pos = 0;
                self.filled_chunk = self.current_chunk;
                self.current_chunk = match self.current_chunk {
                    Chunk::A => {
                        if !self.chunk_b.is_empty() {
                            self.chunk_b.clear();
                        }
                        Chunk::B
                    },
                    Chunk::B => {
                        if !self.chunk_a.is_empty() {
                            self.chunk_a.clear();
                        }
                        Chunk::A},
                    Chunk::None => {Chunk::A} // should never happen
                }
            }
        } else { // if there's not enough space we will need to split the slice
            if slice.len() >= self.chunk_size { // if there's enough to fill a chunk
                if !self.chunk_a.is_empty() {
                    self.chunk_a.clear();
                }
                self.chunk_a.extend_from_slice(&slice[(slice.len()-remaining)..]);
                if !self.chunk_b.is_empty() {
                    self.chunk_b.clear();
                }
                self.filled_chunk = Chunk::A;
                self.current_chunk = Chunk::B;
                self.current_pos = 0;
            } else { // only enough to top off this one and start the next
                let (first, last) = slice.split_at(remaining);
                match self.current_chunk {
                    Chunk::A => {
                        self.chunk_a.extend_from_slice(first);
                        if !self.chunk_b.is_empty() {
                            self.chunk_b.clear();
                        }
                        self.current_pos = last.len();
                        self.chunk_b.extend_from_slice(last);
                        self.filled_chunk = Chunk::A;
                        self.current_chunk = Chunk::B;
                    },
                    Chunk::B => {
                        self.chunk_b.extend_from_slice(first);
                        if !self.chunk_a.is_empty() {
                            self.chunk_a.clear();
                        }
                        self.current_pos = last.len();
                        self.chunk_a.extend_from_slice(last);
                        self.filled_chunk = Chunk::B;
                        self.current_chunk = Chunk::A;
                    },
                    Chunk::None => {} // should never happen
                }
            }
        }
    } //end of fn push_slice
    
}
