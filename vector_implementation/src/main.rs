use std::ptr::NonNull;
use std::mem;
use std::alloc::{self, Layout};

pub struct Vec<T> {
    ptr: NonNull<T>, //指向分配的指针
    len: usize, // 已经初始化的元素个数
    cap: usize, // 分配的内存空间大小
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs"); // if mem::size_of::<T>() == 0, panic: We're not ready to handle ZSTs
        Vec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self) {
        // 计算新的容量大小
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = self.cap * 2;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // 再次检查是否溢出
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");
        
        // 内存分配
        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // 如果分配失败，则终止程序
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(new_layout),
        };

        self.cap = new_cap;
    }

}

fn main() {
    println!("Hello, world!");

}
