pub fn map_box<T, U, F>(boxed: Box<T>, f: F) -> Box<U>
where
    F: Fn(T) -> U,
{
    Box::new(f(*boxed))
}
