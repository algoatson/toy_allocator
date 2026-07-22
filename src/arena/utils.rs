pub(crate) fn align_up(value: usize, align: usize) -> usize {
        debug_assert!(align.is_power_of_two());

        (value + align - 1) & !(align - 1)
    }