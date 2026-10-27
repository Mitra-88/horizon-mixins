pub fn value(input: i32) -> i32 {
    {
        fn replace(input: i32) -> i32 {
            input * 7
        }
        replace(input)
    }
}
