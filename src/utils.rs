#[macro_export]
macro_rules! no_clone_array {
    [$val:expr; $num:expr] => {
        [(); $num].map(|_| $val)
    };
}
