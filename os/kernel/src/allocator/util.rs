/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    check_align( align );
    addr & !( align - 1 )
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    check_align( align );
    if addr & ( align - 1 ) != 0 {
        align_down( addr, align ) + align
    } else {
        addr
    }
}

#[inline]
fn check_align( align: usize ){
    assert!(align.trailing_zeros() > 0 && align.count_ones() == 1 );
}
