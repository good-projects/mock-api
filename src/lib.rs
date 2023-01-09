use std::{
  sync::{mpsc, Arc, Mutex},
  thread,
};

// A thread pool is a group of spawned threads that are waiting and ready to
// handle a task. When the program receives a new task, it assigns one of the
// threads in the pool to the task, and that thread will process the task. The
// remaining threads in the pool are available to handle any other tasks that
// come in while the first thread is processing. When the first thread is done
// processing its task, it’s returned to the pool of idle threads, ready to
// handle a new task.
// A thread pool allows you to process connections concurrently.
pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: Option<mpsc::Sender<Job>>,
}

// Alia for a `Box` that holds the type of closure that execute receives.
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);

    // Use a channel to function as the queue of jobs, the `ThreadPool` will
    // hold on to the `sender` and `execute` will send a job to the `Worker`
    // instances, which will send the job to its thread.
    // Each `Worker` will hold on to the `receiver`.
    let (sender, receiver) = mpsc::channel();

    // The `Arc` type will let multiple workers own the `receiver`, and `Mutex`
    // will ensure that only one worker gets a job from the `receiver` at a
    // time.
    let receiver = Arc::new(Mutex::new(receiver));

    let mut workers = Vec::with_capacity(size);

    // Create the workers and store them in the vector, having them wait for
    // code that we'll send later.
    for id in 0..size {
      workers.push(Worker::new(
        id,
        // Clone the `Arc` to bump the reference count so the workers can share
        // ownership of the `receiver`.
        Arc::clone(&receiver),
      ));
    }

    ThreadPool {
      workers,
      sender: Some(sender),
    }
  }

  pub fn execute<F>(&self, f: F)
  where
    // Use `FnOnce` as the trait, because we'll eventually pass the closure to
    // `thread::spawn` which uses `FnOnce` as the trait too.
    // We can be further confident that `FnOnce` is the trait we want to use
    // because the thread for running a request will only execute that request's
    // closure one time, which matches the _Once_ in `FnOnce`.
    // We also need `Send` to transfer the closure from one thread to another
    // and 'static because we don't know how long the thread will take to
    // execute.
    // `FnOnce()` represents a closure that takes no parameters and returns the
    // unit type (), the return type `()` can be omitted from the signature but
    // not the empty parameter parentheses.
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);

    // Send the `job` down the sending end of the channel.
    // The sending could fail for example if we stop all our threads from
    // executing, meaning the receiving end has stopped receiving new messages.
    // At the moment, we can't stop our threads from executing: our threads
    // continue executing as long as the pool exists. The reason we use unwrap
    // is that we know the failure case won’t happen, but the compiler doesn't
    // know that.
    self.sender.as_ref().unwrap().send(job).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    drop(self.sender.take());

    // Tell the threads they should stop accepting new requests and shut down.
    for worker in &mut self.workers {
      println!("Shutting down worker {}", worker.id);

      // Use `if let` to destructure the `Some` and get the thread.
      // The `take` method on `Option` takes the `Some` variant out and leaves
      // `None` in its place.
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

// The Worker picks up code that needs to be run and runs the code in the
// Worker's thread.
// External code (like our server in src/main.rs) doesn't need to know the
// implementation details regarding using a Worker struct within ThreadPool, so
// we make the Worker struct and its new function private.
struct Worker {
  // This `id` is used to distinguish between the different workers in the pool
  // when logging or debugging.
  id: usize,

  // Use `Option<thread::JoinHandle<()>>`, so we can call the `take` method on
  // the `Option` to move the value out of the `Some` variant and leave a `None`
  // variant in its place. In other words, a `Worker` that is running will have
  // a `Some` variant in thread, and when we want to clean up a `Worker`, we'll
  // replace `Some` with `None` so the `Worker` doesn't have a thread to run.
  //
  // The closures we're passing to the thread pool will not return anything,
  // so we use the unit type `()` as the `T` of `JoinHandle`.
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    // Spawn a JoinHandle<()> instance using an empty closure.
    // If the operating system can't create a thread because there aren't enough
    // system resources, `thread::spawn` will panic. That will cause our whole
    // server to panic, even though the creation of some threads might succeed.
    // TODO(Zhiguang): use `std::thread::Builder` and its spawn method that
    // returns Result instead.
    let thread = thread::spawn(move || loop {
      // Loop forever, asking the receiving end of the channel for a job and
      // running the job when it gets one.
      let message = receiver
        // Call lock on the receiver to acquire the mutex.
        .lock()
        // Call `unwrap` to panic on any errors. Acquiring a lock might fail if
        // the mutex is in a _poisoned_ state, which can happen if some other
        // thread panicked while holding the lock rather than releasing the
        // lock. In this situation, calling `unwrap` to have this thread panic
        // is the correct action to take.
        .unwrap()
        // Call `recv` to receive a `Job` from the channel. If there is no job
        // yet, the current thread will wait until a job becomes available.
        // The `Mutex<T>` ensures that only one `Worker` thread at a time is
        // trying to request a job.
        .recv();

      match message {
        Ok(job) => {
          // println!("Worker {id} got a job; executing.");

          job();
        }
        // Errors might occur if the thread holding the sender has shut down,
        // similar to how the `send` method returns `Err` if the receiver shuts
        // down.
        Err(_) => {
          println!("Worker {id} disconnected; shutting down.");
          break;
        }
      }
    });

    Worker {
      id,
      thread: Some(thread),
    }
  }
}
