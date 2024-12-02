/// Array which will not reallocate any memory, and instead just write to a single array and only change where the array starts.
/// When an item gets pushed while its full it will override existing values.
pub struct LoopArray<T: Default + Copy, const N: usize> {
    // Inclusive
    array_start: usize,
    // Exclusive
    array_stop: usize,
    // If start and stop is in the same index, it could either mean it's empty, or that it's full. This one makes sure we can tell.
    full: bool,
    array: [T; N],
}

impl<T: Default + Copy, const N: usize> LoopArray<T, N> {
    pub fn len(&self) -> usize {
        if self.full {
            return N;
        }
        if self.array_stop < self.array_start {
            return (self.array_stop + N) - (self.array_start);
        } else {
            return self.array_stop - self.array_start;
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        let new_index = (index + self.array_start) % N;
        if new_index >= self.array_stop && new_index < self.array_start {
            panic!("Out of bounds")
        }
        &mut self.array[new_index]
    }

    pub fn push(&mut self, input: T) {
        self.array[self.array_stop] = input;
        self.array_stop = (self.array_stop + 1) % N;
        if self.array_start == self.array_stop {
            self.full = true;
        } else if self.full == true {
            self.array_start = (self.array_start + 1) % N;
        }
    }

    pub fn remove_amount(&mut self, count: usize) {
        if self.len() < count {
            panic!("Removing more elements than what exists")
        } else {
            self.full = false;
            self.array_start = (self.array_start + count) % N;
        }
    }

    pub fn clear(&mut self) {
        self.array_start = 0;
        self.array_stop = 0;
        self.full = false;
    }

    pub fn new() -> LoopArray<T, N> {
        LoopArray {
            array_start: 0,
            array_stop: 0,
            full: false,
            array: [T::default(); N],
        }
    }
}
