#![no_std]

use heapless::HistoryBuffer;

pub struct Debounce<const N: usize> {
    buf: HistoryBuffer<bool, N>,
}

impl<const N: usize> Debounce<N> {
    fn sample(&mut self, input: bool) -> bool {
        self.buf.write(input);
        let mut sum = 0;
        for x in self.buf.oldest_ordered() {
            sum = if *x {
                sum+1
            } else {
                sum-1
            }
        }
        sum > 0
    }
}

impl<const N: usize> Default for Debounce<N> {
    fn default() -> Self {
        Self { buf: HistoryBuffer::new_with(false) }
    }
}

