use multi_impl::multi_impl;

trait AsFloat {
    fn as_float(self) -> f32;
}

multi_impl!(AsFloat for (u32, i32, u64, i64, f64, f32) {
    fn as_float(self) -> f32 {
        self as f32
    }
});

#[test] 
fn test() {
    let float = 5.as_float();

    assert_eq!(float, 5.0);
}