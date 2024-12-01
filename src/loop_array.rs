/// Array which will not reallocate any memory, and instead just write to a single array and only change where the array starts.
/// When an item gets pushed while its full it will override existing values.
pub struct LoopArray<T: Default + Copy, const N: usize> {
    array_start: usize,
    array_stop: usize,
    array: [T; N],
}

impl<T: Default + Copy, const N: usize> LoopArray<T, N> {
    pub fn len(&self) -> usize {
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
    }

    pub fn remove_amount(&mut self, count: usize) {
        if self.len() < count {
            panic!("Removing more elements than what exists")
        } else {
            self.array_start = (self.array_start + count) % N;
        }
    }

    pub fn new() -> LoopArray<T, N> {
        LoopArray {
            array_start: 0,
            array_stop: 0,
            array: [T::default(); N],
        }
    }
}
