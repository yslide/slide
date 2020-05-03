pub fn print<T: Print>(obj: T) -> String {
    obj.print()
}

pub trait Print {
    fn print(self) -> String;
}
