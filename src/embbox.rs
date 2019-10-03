//use core::ops::{Generator, GeneratorState};
//use core::pin::Pin;
//
//struct Valve {
//    pub reg : u32,
//}
//
//    rtt_print!("Test 1");
//
//    let mut v = Valve { reg : 0, };
//
//    let mut sb : EmbBox< dyn Generator<Yield = u32, Return = !> + core::marker::Unpin, [usize; 8]> = embbox!{
//        || {
//            
//            loop {
//                yield 0u32;
//                v.reg = 2;
//                yield 1;
//                v.reg = 3;
//                yield 2;
//            }
//        }
//    };
//
//    rtt_print!("Test 2");
//    for _ in 0 .. 100 {
//        match Pin::new(&mut *sb).resume() {
//            GeneratorState::Yielded(num) => { rtt_print!("Step : {}", num); }
//            GeneratorState::Complete(_) => { rtt_print!("Finish step!"); }
//            _ => panic!("unexpected value from resume"),
//        }
//    }
//

#![allow(dead_code)]

use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop};
use core::ops;
use core::ptr;

#[macro_export]
macro_rules! embbox {
    ( $e: expr ) => {{
        let val = $e;
        let ptr = &val as *const _;
        #[allow(unsafe_code)]
        unsafe {
            EmbBox::new_unchecked(val, ptr)
        }
    }};
}

pub struct EmbBox<T: ?Sized, Space> {
    space: ManuallyDrop<Space>,
    ptr: *const T,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, Space> EmbBox<T, Space> {

    #[inline(always)]
    pub fn new(val: T) -> EmbBox<T, Space>
    where
        T: Sized,
    {
        embbox!(val)
    }

    #[inline]
    pub unsafe fn new_unchecked<U>(val: U, ptr: *const T) -> EmbBox<T, Space>
    where
        U: Sized,
    {
        let result = Self::new_copy(&val, ptr);
        mem::forget(val);
        result
    }

    unsafe fn new_copy<U>(val: &U, ptr: *const T) -> EmbBox<T, Space>
    where
        U: ?Sized,
    {
        let size = mem::size_of_val::<U>(val);
        let align = mem::align_of_val::<U>(val);

        let mut space = ManuallyDrop::new(mem::uninitialized::<Space>());

        let (ptr_addr, ptr_copy): (*const u8, *mut u8) = if size == 0 {
            (ptr::null(), align as *mut u8)
        } else if size > mem::size_of::<Space>() || align > mem::align_of::<Space>() {
            // Heap
            panic!("Can not fit value into storage! storage size {} < value size {} OR storage align {} < value align {}",
                mem::size_of::<Space>(), size, mem::align_of::<Space>(), align);
        } else {
            // Stack
            (ptr::null(), mem::transmute(&mut space))
        };

        let mut ptr = ptr;
        let ptr_ptr = &mut ptr as *mut _ as *mut usize;
        ptr_ptr.write(ptr_addr as usize);

        ptr::copy_nonoverlapping(val as *const _ as *const u8, ptr_copy, size);

        EmbBox {
            space,
            ptr,
            _phantom: PhantomData,
        }
    }

    #[inline]
    unsafe fn as_ptr(&self) -> *const T {
        let mut ptr = self.ptr;

        let ptr_ptr = &mut ptr as *mut _ as *mut usize;
        ptr_ptr.write(mem::transmute(&self.space));

        ptr
    }
}

impl<T: ?Sized, Space> ops::Deref for EmbBox<T, Space> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized, Space> ops::DerefMut for EmbBox<T, Space> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_ptr() as *const _ as *mut _) }
    }
}

impl<T: ?Sized, Space> ops::Drop for EmbBox<T, Space> {
    fn drop(&mut self) {
        unsafe {
            //let layout = Layout::for_value::<T>(&*self);
            ptr::drop_in_place::<T>(&mut **self);
            //if self.is_heap() {
            //    alloc::dealloc(self.ptr as *mut u8, layout);
            //}
        }
    }
}