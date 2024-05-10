use std::ptr::NonNull;

pub struct Vec<T> {
    ptr: NonNull<T>, //指向分配的指针
    len: usize, // 已经初始化的元素个数
    cap: usize, // 分配的内存空间大小
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

fn main() {
    println!("Hello, world!");
}
