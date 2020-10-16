

macro_rules! mysql_row_try_get {
    // `()` indicates that the macro takes no argument.
    ($var: tt, $index: tt) => {
        // The macro will expand into the contents of this block.
        $var.get_opt($index).unwrap()
    };
}
