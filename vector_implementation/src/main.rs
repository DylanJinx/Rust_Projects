use std::ptr::{self, NonNull};
use std::mem;
use std::alloc::{self, Layout};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct MyVec<T> {
    ptr: NonNull<T>, //指向分配的指针
    len: usize, // 已经初始化的元素个数
    cap: usize, // 分配的内存空间大小
}

unsafe impl<T: Send> Send for MyVec<T> {}
unsafe impl<T: Sync> Sync for MyVec<T> {}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs"); // if mem::size_of::<T>() == 0, panic: We're not ready to handle ZSTs
        MyVec {
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

    pub fn insert(&mut self, index: usize, elem: T) {
        // 检查索引是否越界, 0 <= index <= self.len
        assert!(index <= self.len, "index out of bounds");
        
        // 如果当前元素个数等于容量，需要扩容
        if self.len == self.cap { self.grow(); }

        unsafe {
            // ptr::copy(src, dest, len) 的含义： "从 src 复制连续的 len 个元素到 dest "
            ptr::copy(
                self.ptr.as_ptr().add(index), // 将指针从指向数组的首元素移动到索引为 index 的元素
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.as_ptr().add(index), elem);
        }

        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        // 检查索引是否越界, 0 <= index < self.len
        assert!(index < self.len, "index out of bounds");

        let remove_elem;

        unsafe {
            self.len -= 1;
            remove_elem = ptr::read(self.ptr.as_ptr().add(index));
            ptr::copy(
                self.ptr.as_ptr().add(index + 1),
                self.ptr.as_ptr().add(index),
                self.len - index,
            );
        }

        remove_elem
    }

}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            println!("要开始释放内存咯！");
            while let Some(_) = self.pop() { }
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> Deref for MyVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for MyVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

fn main() {
    println!("----------------------------------------12345. new、push、pop、drop、deref ------------------------------------");
    let mut v = MyVec::new(); // 创建一个空的 MyVec

    // 添加元素
    v.push(1);
    v.push(2);
    v.push(3);

    // let v1 = &*v; //type v1 = &[i32]

    // 打印当前的 MyVec
    // 使用解引用来获取 MyVec 的 slice,*v 首先对 v 进行解引用，通过 Deref 特质将 v 转换为它的目标类型，即 Vec<T> 的目标类型 [T]（切片）。然后，& 对解引用得到的值再次取引用，最终结果是一个指向切片的引用。由于 [T] 类型实现了 Debug 特质，所以这里可以使用 {:?} 格式化打印切片内容。
    println!("Current MyVec: {:?}", &*v); // Current MyVec: [1, 2, 3]
    println!("Current MyVec: {:?}", v); // Current MyVec: MyVec { ptr: 0x250832bc770, len: 3, cap: 4 }

    // 移除一个元素
    if let Some(value) = v.pop() {
        println!("Popped value: {}", value);
    }

    // 再次打印 MyVec 查看变化
    println!("MyVec after pop: {:?}", &*v); // MyVec after pop: [1, 2]
    println!("MyVec after pop: {:?}", v); // MyVec after pop: MyVec { ptr: 0x250832bc770, len: 2, cap: 4 }

    // 尝试继续添加和移除元素来测试动态扩展
    v.push(4);
    v.push(5);

    println!("MyVec after adding more elements: {:?}", &*v); // MyVec after pop: [1, 2, 4, 5]
    println!("MyVec after adding more elements: {:?}", v); // MyVec after adding more elements: MyVec { ptr: 0x250832bc770, len: 4, cap: 4 }

    // 清空 MyVec 并检查是否正确释放内存
    while let Some(value) = v.pop() {
        println!("Popping: {}", value);
    }
    
    // 最终状态
    println!("Final MyVec: {:?}", &*v); // Final MyVec: []
    println!("Final MyVec: {:?}", v);  // Final MyVec: MyVec { ptr: 0x250832bc770, len: 0, cap: 4 }

    let mut inter_v = MyVec::new();
    inter_v.push("hello".to_string());
    inter_v.push(",".to_string());
    inter_v.push("world".to_string());
    inter_v.push("!".to_string());

    // 使用索引访问
    //当使用 v[2] 这样的索引访问时，实际上是通过 Deref 特质自动解引用到 [T]，然后在这个切片上进行索引访问。这是因为 [] 操作符是通过 Index 或 IndexMut 特质实现的，而这些特质在 [T] 类型上已经由标准库实现。
    // let a = inter_v[2].clone(); // type a = String
    println!("inter_v[3]: {}", inter_v[3]);  // 直接使用索引访问，如同数组

    // 直接在MyVec上迭代
    // &*v 首先使用 * 解引用 v 到 [T] 类型的切片，然后 & 重新取得这个切片的不可变引用。这让整个结构变为 &[T]，是一个指向切片的引用，可以直接在 for 循环中迭代。
    // 这里的解引用是通过 Deref 特质实现的，这种机制允许 MyVec<T> 的实例直接访问 [T] 上定义的方法和特质实现，包括用于迭代的 IntoIterator。
    let mut i = 0;
    for item in &*inter_v {       
        println!("{i} : {}", item);
        i = i + 1;
    }

    println!("Before drop: {:?}", &*inter_v);
    println!("Before drop: {:?}", inter_v);

    // 手动释放内存
    drop(inter_v);
    // println!("After drop: {:?}", &*inter_v); // inter_v 已经被 drop 了，这里会报错
    // println!("After drop: {:?}", inter_v); // inter_v 已经被 drop 了，这里会报错

    println!("-----------------------------------------------6. insert and remove -----------------------------------------------");
    // 插入和删除元素
    let mut v_6 = MyVec::new();
    v_6.push(1);
    v_6.push(2);
    v_6.push(3);
    v_6.push(4);

    println!("Before insert: {:?}", &*v_6); // Before insert: [1, 2, 3, 4]
    println!("Before insert: {:?}", v_6); //After insert: MyVec { ptr: 0x2259113c810, len: 4, cap: 4 }
    
    v_6.insert(2, 10);
    println!("After insert: {:?}", &*v_6); // After insert: [1, 2, 10, 3, 4]
    println!("After insert: {:?}", v_6); //After insert: MyVec { ptr: 0x2259113c810, len: 5, cap: 8 }，扩容了

    v_6.remove(1);
    println!("After remove: {:?}", &*v_6); // After remove: [1, 10, 3, 4]
    println!("After remove: {:?}", v_6); //After remove: MyVec { ptr: 0x2259113c810, len: 4, cap: 8 }，因为没有使用调用了如 shrink_to_fit 的方法，所以不会缩减 Vec 的内存容量

}
