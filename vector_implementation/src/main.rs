use std::ptr::{self, NonNull};
use std::mem::{self, ManuallyDrop};
use std::alloc::{self, Layout};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        // 暂时不支持 ZST：Zero Sized Type
        assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");

        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    fn grow(&mut self) {
        // 保证新申请的内存没有超出 isize 的最大值
        let new_cap = if self.cap == 0 {
            1
        } else {
            self.cap * 2
        };

        // `Layout::array` 会检查申请的空间是否小于等于 usize::MAX，
        // 但是因为 old_layout.size() <= isize::MAX，
        // 所以这里的 unwrap 永远不可能失败
        let new_layout = Layout::array::<T>(new_cap).unwrap(); 

        // 保证新申请的内存没有超出 `isize::MAX` 字节
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // 如果分配失败，`new_ptr` 就会成为空指针，我们需要对应 abort 的操作
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            println!("RawVec要开始释放内存咯！");
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    
    }
}

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    unsafe fn new(slice: &MyVec<T>) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if slice.cap() == 0 {
                slice.ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}

// 向前迭代
impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(result)
            }
        }
    }

    // size_hint 方法返回一个元组，包含了迭代器的最小和最大元素数量的估计值。这个方法是为了帮助标准库中的一些方法进行性能优化的，例如 Vec 的 iter() 方法会根据 size_hint 的返回值来决定是否使用 memcpy 来提高性能。
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}

// 向后迭代
impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

#[derive(Debug)]
pub struct MyVec<T> {
    buf: RawVec<T>, // 指向分配的内存
    // ptr: NonNull<T>, //指向分配的指针
    // cap: usize, // 分配的内存空间大小
    len: usize, // 已经初始化的元素个数
}

unsafe impl<T: Send> Send for MyVec<T> {}
unsafe impl<T: Sync> Sync for MyVec<T> {}

impl<T> MyVec<T> {
    // 将push/pop/insert/remove中的self.ptr.as_ptr() 变成 调用方法 self.ptr()
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    // 将push/pop/insert/remove中的self.cap 变成 调用方法 self.cap()
    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Self {
        MyVec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    // MyVec<T> 的 self.grow() 现在由 RawVec<T> 的 self.buf.grow() 实现
    //fn grow(&mut self) { ... }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() { self.buf.grow();}

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
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
                Some(ptr::read(self.ptr().add(self.len)))
            }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        // 检查索引是否越界, 0 <= index <= self.len
        assert!(index <= self.len, "index out of bounds");
        
        // 如果当前元素个数等于容量，需要扩容
        if self.len == self.cap() { self.buf.grow(); }

        unsafe {
            // ptr::copy(src, dest, len) 的含义： "从 src 复制连续的 len 个元素到 dest "
            ptr::copy(
                self.ptr().add(index), // 将指针从指向数组的首元素移动到索引为 index 的元素
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
        }

        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        // 检查索引是否越界, 0 <= index < self.len
        assert!(index < self.len, "index out of bounds");

        let remove_elem;

        unsafe {
            self.len -= 1;
            remove_elem = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
        }

        remove_elem
    }

    pub fn drain(&mut self) -> Drain<T> {
        let iter = unsafe {RawValIter::new(&self)};

        // 通过将长度设为 0，Vec<T> 本质上不再认为自己拥有任何元素。即使其底层内存（由 RawVec<T> 管理）还没有被释放，Vec<T> 也不会在其生命周期结束时尝试释放这些内存。这是因为只有当长度大于 0 时，Vec<T> 的析构函数才会处理内存释放。
        // 避免双重释放
        // 虽然Drain可能会mem::forget，导致内存泄露，但是这个错误比双重释放要轻，这通常被视为相对安全的失败模式
        self.len = 0;

        Drain {
            iter : iter,
            vec: PhantomData,
        }
    }

}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        if self.cap() != 0 {
            println!("MyVec要开始释放内存咯！");
            let mut i = 0;
            while let Some(_) = self.pop() {
                println!("MyVec.drop pop: {:?}", i);
                i += 1;
             }
            // 释放内存的操作将由 RawVec 的 Drop 负责
                // let layout = Layout::array::<T>(self.cap()).unwrap();
                // unsafe {
                //     alloc::dealloc(self.ptr() as *mut u8, layout);
                // }
        }
    }
}

//Deref 特质使对象能够通过 * 运算符被解引用，并且能够隐式地转换为一个引用指向的类型。当你通过 & 符号取得一个实现了 Deref 特质的类型的引用时，Rust 会自动地使用 Deref 的实现来将 &Type 转换为 &Target
impl<T> Deref for MyVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr(), self.len) // 由self.ptr.as_ptr() 变成 调用方法 self.ptr()
        }
    }
}

// DerefMut 特质使对象能够通过 * 运算符被解引用为可变引用，并支持类型的自动转换。实现了 DerefMut 的对象在使用 &mut 获取引用时，可以自动转换为 &mut Target
impl<T> DerefMut for MyVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr(), self.len) // 由self.ptr.as_ptr() 变成 调用方法 self.ptr()
        }
    }
}


pub struct MyIntoIter<T> {
    _buf: RawVec<T>, // 实际上并不关心这个，只需要他们保证分配的空间不被释放
        // buf: NonNull<T>,
        // cap: usize,
    iter: RawValIter<T>,
        // start: *const T,
        // end: *const T,  
}

impl<T> IntoIterator for MyVec<T> {
    type Item = T;
    type IntoIter = MyIntoIter<T>;
    fn into_iter(self) -> MyIntoIter<T> {
        // 需要使用 ptr::read 非安全地把 buf 移出，因为它没有实现 Copy，
        // 而且 Vec 实现了 Drop Trait (因此我们不能销毁它)
        let buf = unsafe { ptr::read(&self.buf) };
        let iter = unsafe { RawValIter::new(&self) };

        mem::forget(self); // 避免调用 drop 方法

        MyIntoIter {
            iter: iter,
            //cap,
            // start: buf.ptr.as_ptr(),
            // end: if buf.cap == 0 {
            //     buf.ptr.as_ptr()
            // } else {
            //     unsafe { buf.ptr.as_ptr().add(len) }
            // },
            _buf: buf, //_buf的赋值要在start和end之后，因为start和end是从buf中获取的；这里的赋值会move buf的所有权
        }
    }
}

// 向前迭代
impl<T> Iterator for MyIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    // size_hint 方法返回一个元组，包含了迭代器的最小和最大元素数量的估计值。这个方法是为了帮助标准库中的一些方法进行性能优化的，例如 Vec 的 iter() 方法会根据 size_hint 的返回值来决定是否使用 memcpy 来提高性能。
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// 向后迭代
impl<T> DoubleEndedIterator for MyIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

// MyIntoIter_Drop
impl<T> Drop for MyIntoIter<T> {
    fn drop(&mut self) {
        if self._buf.cap != 0 {
            // 将剩下的元素drop
            //println!("MyIntoIter要开始释放内存咯！");
            //我们只需要确保 Vec 中所有元素都被读取了，
            for _ in &mut *self { println!("MyIntoIter正在释放未读取元素的内存！");}

            println!("MyIntoIter已经释放所有元素的内存！");

            // 释放内存的操作将由 RawVec 的 Drop 负责
                // let layout = Layout::array::<T>(self._buf.cap).unwrap();
                // unsafe {
                // alloc::dealloc(self._buf.ptr.as_ptr() as *mut u8, layout);
                // }
        }
    }
}


struct Drain<'a, T> {
    vec: PhantomData<&'a mut Vec<T>>,
    iter: RawValIter<T>,
        // start: usize,
        // end: usize,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> { self.iter.next_back() }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {
            println!("Drain正在释放未读取元素的内存！");
        }
        println!("Drain已经释放所有元素的内存！");
    }
}

fn main() {
    println!("----------------------------------------12345. new、push、pop、drop、deref ------------------------------------");
    
    println!("---------------------------------------- new、push、pop ------------------------------------");
    { 
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

    }

    println!("--------------------------------------- deref、drop ----------------------------------------");
    {   
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
        // 这里的解引用是通过 Deref 特质实现的，这种机制允许 MyVec<T> 的实例直接访问 [T] 上定义的方法和特质实现，包括用于迭代的 iter()、iter_mut()、into_iter()，因为[T]实现了IntoIterator特质。
        let mut i = 0;
        for item in inter_v.iter() {       
            println!("{i} : {}", item);
            i = i + 1;
        }
        println!("inter_v use iter() : {:?}", &*inter_v);
        println!("inter_v use iter() : {:?}", inter_v);

        let mut i = 0;
        for item in inter_v.iter_mut() {       
            println!("{i} : {}", item);
            i = i + 1;
        }
        println!("inter_v use iter_mut() : {:?}", &*inter_v);
        println!("inter_v use iter_mut() : {:?}", inter_v);

        let mut i = 0;
        for item in inter_v.into_iter() {       
            println!("{i} : {}", item);
            i = i + 1;
        }
        //println!("inter_v use into_iter() : {:?}", &*inter_v); //move
        //println!("inter_v use into_iter() : {:?}", inter_v); //move

        let mut deref_v = MyVec::new();
        deref_v.push("hello".to_string());
        deref_v.push(",".to_string());
        deref_v.push("world".to_string());
        deref_v.push("!".to_string());
        // 直接使用&[T]
        let mut i = 0;
        for item in &*deref_v {       
            println!("{i} : {}", item);
            i = i + 1;
        }

        println!("deref_v : {:?}", &*deref_v);
        println!("deref_v : {:?}", deref_v);

        // 手动释放内存
        drop(deref_v);
        // println!("After drop: {:?}", &*inter_v); // inter_v 已经被 drop 了，这里会报错
        // println!("After drop: {:?}", inter_v); // inter_v 已经被 drop 了，这里会报错

    }
    
    println!("-----------------------------------------------6. insert and remove -----------------------------------------------");
    {
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

    println!("-----------------------------------------------7. IntoIter -----------------------------------------------");
    {
        {
            println!("--------------------遍历全部-------------------");
            let mut v_7 = MyVec::new();
            v_7.push("hello".to_string());
            v_7.push(",".to_string());
            v_7.push("world".to_string());
            v_7.push("!".to_string());

            let v_7_iter = v_7.into_iter();
            for item in v_7_iter {
                println!("{}", item);
                // if item == "world".to_string() {
                //     break;
                // }
            }
        }

        {
            println!("--------------------不遍历全部-------------------");
            let mut v_7 = MyVec::new();
            v_7.push("hello".to_string());
            v_7.push(",".to_string());
            v_7.push("world".to_string());
            v_7.push("!".to_string());

            let v_7_iter = v_7.into_iter();
            for item in v_7_iter {
                println!("{}", item);
                if item == "world".to_string() {
                    break;
                }
            }
        }
    }

    println!("-----------------------------------------------9. Drain -----------------------------------------------");
    {

        {
            println!("--------------------遍历全部-------------------");
            let mut v_9 = MyVec::new();
            v_9.push("hello".to_string());
            v_9.push(",".to_string());
            v_9.push("world".to_string());
            v_9.push("!".to_string());

            let mut drain = v_9.drain();
            for item in &mut drain {
                println!("{}", item);
                // if item == "world".to_string() {
                //     break;
                // }
            }
        }
            
        {
            println!("--------------------不遍历全部-------------------");
            let mut v_9 = MyVec::new();
            v_9.push("hello".to_string());
            v_9.push(",".to_string());
            v_9.push("world".to_string());
            v_9.push("!".to_string());

            let mut drain = v_9.drain();
            for item in &mut drain {
                println!("{}", item);
                if item == "world".to_string() {
                    break;
                }
            }

            //当 `Drain` 结构体的 `drop` 函数被调用时，它会通过 `next()` 方法继续遍历剩余未被处理的元素。在你的代码中，如果迭代在处理 `'world'` 之后中断，那么 `Drain` 的 `drop` 方法会继续处理 `'!'` 元素。这个过程发生在 `Drain` 结构体生命周期结束时，即在其作用域结束时自动调用 `drop`。
            //因此，当控制流回到 `MyVec<T>` 的 `drop` 函数时，`len` 已经被设置为 0（因为 `Drain` 的开始就将 `len` 设置为 0），所以 `MyVec<T>` 的 `drop` 函数中调用的 `pop` 方法实际上不会执行任何操作，因为它会立即因为 `len` 是 0 而返回 `None`。这意味着 `MyVec<T>` 中已经没有元素可以 `pop`，因为所有元素都已经在 `Drain` 中被处理了。这种设计确保了即使部分元素没有在主循环中被显式消费，也不会发生内存泄漏，同时也防止了双重释放的问题。
        }
    }
}
