use std::cell::RefCell;

#[derive(Debug)]
struct SimpleStack<T>{
    stack: RefCell<Vec<T>>,
}

impl<T> SimpleStack<T> {
    fn new() -> SimpleStack<T>{
        SimpleStack { 
            stack: RefCell::new(Vec::new()) 
        }
    }

    fn push(&self,value:T){
        self.stack.borrow_mut().push(value);
    }

    fn pop(&self)->Option<T>{
        self.stack.borrow_mut().pop() //return option
    }
}

fn main(){
    let stack = SimpleStack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);

    println!("Popped value: {:?}",stack.pop()); //3
    println!("Popped value: {:?}",stack.pop()); //2

    stack.push(4);

    println!("Popped value: {:?}",stack.pop()); //4
    println!("Popped value: {:?}",stack.pop()); //1
    println!("Popped value: {:?}",stack.pop()); //none
}