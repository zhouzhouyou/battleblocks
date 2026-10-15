//! Snapshot-friendly random stream shared by board refill and AI.
use std::cell::Cell;
thread_local! { static STATE:Cell<u64> = const { Cell::new(0x9e3779b97f4a7c15) }; }
pub fn state() -> u64 {
    STATE.with(Cell::get)
}
pub fn restore(value: u64) {
    STATE.with(|s| s.set(value));
}
pub fn srand(seed: u64) {
    restore(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed });
}
fn next() -> u64 {
    let mut v = state();
    v ^= v >> 12;
    v ^= v << 25;
    v ^= v >> 27;
    restore(v);
    v.wrapping_mul(2685821657736338717)
}
pub trait Integer: Copy {
    fn number(self) -> u64;
    fn from_number(v: u64) -> Self;
}
macro_rules! integers {($($t:ty),*)=>{$(impl Integer for $t {fn number(self)->u64{self as u64}fn from_number(v:u64)->Self{v as Self}})*};}
integers!(u8, usize, i32, u64);
pub fn gen_range<T: Integer>(low: T, high: T) -> T {
    let lo = low.number();
    let width = high
        .number()
        .checked_sub(lo)
        .filter(|&w| w > 0)
        .expect("Invalid random range");
    // Rejection avoids modulo bias.
    let limit = u64::MAX - u64::MAX % width;
    loop {
        let value = next();
        if value < limit {
            return T::from_number(lo + value % width);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restore_reproduces_refill() {
        srand(42);
        let checkpoint = state();
        let first: Vec<u8> = (0..64).map(|_| gen_range(0, 7)).collect();
        restore(checkpoint);
        let second: Vec<u8> = (0..64).map(|_| gen_range(0, 7)).collect();
        assert_eq!(first, second);
        assert!(first.iter().all(|&v| v < 7));
    }
}
