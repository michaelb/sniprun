use std::sync::{mpsc, Arc, Mutex};
use std::thread;

enum HandleAction {
    New(thread::JoinHandle<()>),
    Kill,
}

fn main() {
    let some_stuff = Arc::new(Mutex::new(String::from("lol"))); // my type does not impl Copy

    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut handle: Option<thread::JoinHandle<()>> = None;
        loop {
            match recv.recv() {
                Err(_) => panic!("Idk"),
                Ok(HandleAction::Kill) => {
                    if let Some(handle) = handle.take() {
                        handle.join().unwrap()
                    }
                }
                Ok(HandleAction::New(new)) => handle = Some(new),
            }
        }
    });

    for message in 1..5500 {
        //pretend it's my receiver infinite loop
        println!("{}", message);
        match message % 2 {
            // pretend even means 'run' and odd means 'terminate'
            0 => {
                //Run
                //handle.stop;
                let stuff = some_stuff.clone();
                send.send(HandleAction::New(thread::spawn(move || {
                    //special thread i can kill, crate: stoppable-thread
                    stuff.lock().unwrap().truncate(2); //need to call method that need mut
                })))
                .unwrap();
            }
            _ => send.send(HandleAction::Kill).unwrap(), // in reality, handle.stop() :terminate
        }
    }
}
