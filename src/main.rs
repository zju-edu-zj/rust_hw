use std::cell::RefCell;
use std::future::Future;
use std::sync::Condvar;
use std::task::{Waker, Wake, Context, Poll};
use std::time::Duration;
//use std::thread::spawn;
use async_channel;
use futures::future::BoxFuture;
use scoped_tls::scoped_thread_local;
use std::collections::VecDeque;
use std::sync::{Arc,Mutex};
//use std::time::Duration;

struct Signal{
    state: Mutex<State>, //use state to control condvar
    cond: Condvar,
}
enum State{
    Empty, //the initial state
    Waiting,  //wait for wake
    Notified,
}
impl Signal{
    fn new()->Self{
        Signal {
            state: Mutex::new(State::Empty), 
            cond: Condvar::new()
        }
    }
    fn wait(&self){
        let mut state = self.state.lock().unwrap(); //get control and change the state
        match *state{
            State::Notified => *state = State::Empty,
            State::Waiting =>{
                panic!("multiple wait");
            }
            State::Empty=>{
                *state = State::Waiting;
                while let State::Waiting = *state{
                    state = self.cond.wait(state).unwrap(); //wait for notifying
                    //println!("?");
                }
            }
        }
    }
    fn notify(&self){
        let mut state = self.state.lock().unwrap();
        match *state{
            State::Notified =>{}
            State::Empty => *state = State::Notified,
            State::Waiting => {
                *state = State::Empty;
                self.cond.notify_one(); //wake function
            }
        }
    }
}
impl Wake for Signal{
    fn wake(self:Arc<Self>){
        self.notify();
    }
}

scoped_thread_local!(static RUNNABLE: Mutex<VecDeque<Arc<Task>>>); //create a global varible
scoped_thread_local!(static SIGNAL: Arc<Signal>);

/// include future and signal which can wake itself
struct Task{
    future: RefCell<BoxFuture<'static,()>>,
    signal: Arc<Signal>,
}
unsafe impl Send for Task {    }
unsafe impl Sync for Task {    }
impl Wake for Task {
    fn wake(self:Arc<Self>){
        RUNNABLE.with(|runnable| runnable.lock().unwrap().push_back(self.clone())); //push to the vecdeque whihc can be executed
        self.signal.notify();
    }
}

/// a simple runtime which includes function "spawn" to create another future and block_on to run future
struct RunTime;
impl RunTime{
    fn spawn<F: Future<Output = ()> +'static + Send>(future: F) {
        let signal = Arc::new(Signal::new()); //create waker again
        let waker = Waker::from(signal.clone());
        let task = Arc::new(Task {
            future: RefCell::new(Box::pin(future)),
            signal: signal.clone(),
        }); //pack to a task
        let mut cx = Context::from_waker(&waker);
        if let Poll::Ready(_) = task.future.borrow_mut().as_mut().poll(&mut cx) {
            return; //no result
        }
        
        RUNNABLE.with(|runnable| {
            runnable.lock().unwrap().push_back(task); //to execute the son task
            //signal.notify();
        })
    }
    
    fn block_on<F:Future>(future:F)->F::Output{
        let mut fut = std::pin::pin!(future);
        let signal = Arc::new(Signal::new());
        let waker = Waker::from(signal.clone());
        //let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        let runnable = Mutex::new(VecDeque::with_capacity(1024));
        SIGNAL.set(&signal,||{
            RUNNABLE.set(&runnable,||loop{    //the loop is the main procedure 
                if let Poll::Ready(output) = fut.as_mut().poll(&mut cx){
                    return output;  //the main task is over
                }
                while let Some(task) = runnable.lock().unwrap().pop_front() { //execute son tasks continuously
                    let waker = Waker::from(task.clone());
                    let mut cx = Context::from_waker(&waker);
                    let _ = task.future.borrow_mut().as_mut().poll(&mut cx);
                }
                signal.wait(); //we just wait for notifying and do nothing
            })
        })
    }
}



async fn demo1(){
    let (tx,rx) = async_channel::bounded::<()>(1);
    RunTime::spawn(demo2(tx)); //create a new future
    let _ = rx.recv().await; //wait for sending
    println!("I'm demo1");
}
async fn demo2(tx:async_channel::Sender<()>){
    println!("I'm demo2"); //demo2 is supposed to be shown frist
    std::thread::sleep(Duration::from_secs(2)); //sleep for 2 seconds
    let _ = tx.send(()).await;
}

fn main(){
    RunTime::block_on(demo1()); //simply run the main future
}