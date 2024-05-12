use std::ptr::{self, NonNull};
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

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap { self.grow();}

        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }

        // 不可能出错，因为出错之前一定会 OOM (out of memory)
        // 如果 `self.grow()` 不能成功扩展内存（可能因为内存耗尽），程序将在那一步失败。因此，在 `ptr::write` 之后增加 `self.len` 被认为是安全的，因为内存空间已经通过 `grow` 方法保证了
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.as_ptr().add(self.len)))
            }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap !=0 {
            while let Some(_) = self.pop() { }
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

fn main() {
    println!("Hello, world!");

}
