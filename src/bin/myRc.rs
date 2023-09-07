
use std::cell::UnsafeCell;
use std::ops::Deref;

struct MyRc<T> {
    data: *mut UnsafeCell<(T, usize)>  //孩子实在没办法了
}

impl<T> MyRc<T> {
    fn new(value: T) -> MyRc<T> {
        MyRc {
            data: Box::leak(Box::new(UnsafeCell::new((value, 1)))) //from box we obtain a pointer
        }
    }
    fn strong_count(&self) -> usize {
        unsafe {
            (*(*self.data).get()).1 as usize
        }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &(*(*self.data).get()).0  //return a ref
        }
    }
}

impl<T> Clone for MyRc<T> {
    fn clone(&self) -> MyRc<T> {
        println!("count before clone: {}", self.strong_count());
        unsafe {
            (*(*self.data).get()).1 += 1 ; //increase by 1 each time
        }
        MyRc {
            data: self.data //return the pointer directly
        }
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        unsafe {
            (*(*self.data).get()).1 -= 1 ; //decrase by 1
        }
        
        println!("count after drop once: {}", self.strong_count());
        
        if self.strong_count() == 0 {
            let _a = self.data; //free the memory
        }
    }
}

fn main() {
    let rc1 = MyRc::new(42);
    let rc2 = rc1.clone();

    println!("Count after clone rc2: {}", rc1.strong_count()); //should be 2
    assert_eq!(rc2.strong_count(),2); //be 2 too
}
