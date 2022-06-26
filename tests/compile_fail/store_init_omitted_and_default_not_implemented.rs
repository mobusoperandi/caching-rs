use michie::{memoized, MemoizationStore};

struct Store;
impl MemoizationStore<usize, usize> for Store {
    fn insert(&mut self, _input: usize, return_value: usize) -> usize {
        return_value
    }
    fn get(&self, _input: &usize) -> Option<usize> {
        None
    }
}
#[memoized(key_expr = input, store_type = Store)]
fn f(input: usize) -> usize {
    input
}

fn main() {}
