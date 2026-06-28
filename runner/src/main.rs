use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread};
use std::io;
use std::env;
mod universe;

fn run(uname: &str)
{
 println!("Universe {:?}", uname);

 let cancel_request = Arc::new(AtomicBool::new(false));
 let cancel_ticket = cancel_request.clone();
 let fname = uname.to_owned();

 println!("Press ENTER to stop");
 let unith = thread::spawn(move || universe::Universe::run(fname, cancel_ticket));

 let mut input = String::new();
 io::stdin().read_line(&mut input).expect("Failed to read");
 cancel_request.store(true, Ordering::Relaxed);
 unith.join().unwrap();
}

fn main()
{
 let args: Vec<String> = env::args().collect();

 match args.len()
 {
  2 => { run(&args[1]); },
  _ => { println!("Usage {:?} filename", &args[0]); }
 }
}
