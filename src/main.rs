pub mod user_message;

use std::{net::{TcpListener, TcpStream}, sync::{Arc, mpsc::{Sender, Receiver, self}, RwLock}, io::{Write, BufReader, BufRead}, error::Error, thread};

use user_message::UserMessage;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:5000").expect("could not bind server");

    let (tx, rx): (Sender<UserMessage>, Receiver<UserMessage>) = mpsc::channel();
    let users: Arc<RwLock<Vec<Arc<TcpStream>>>> = Arc::new(RwLock::new(Vec::new()));

    let temp_users = Arc::clone(&users);

    thread::spawn(move || {
        echo_messages(rx, &temp_users)
    });

    for stream in listener.incoming() {
        let stream = Arc::new(stream.expect("Could not connect incoming stream"));
        users.write().unwrap().push(Arc::clone(&stream));
        let msg_send_clone = tx.clone();
        let thread_handle = thread::spawn(move || { 
            match handle_user(stream, msg_send_clone) {
                Ok(_) => println!("A client disconnected successfully!"),
                Err(err) => {
                    println!("{}", err);
                }
            }
        });
        if let Err(_) = thread_handle.join() {
            println!("Thread panic!");
        }
    }
}

fn handle_user(user: Arc<TcpStream>, msg_chan: Sender<UserMessage>) -> Result<(), Box<dyn Error>> {

    (&*user).write_all("~ASK_NAME~\x04".as_bytes())?;
    
    let mut stream_reader = BufReader::new(&*user);
    let mut recv: Vec<u8> = Vec::new();
    if matches!(stream_reader.read_until(0x04, &mut recv), Ok(0)) {
        println!("Unnamed client disconnected");
        return Ok(());
    }
    let name = String::from_utf8(recv.clone())?;

    loop {
        recv.clear();
        if let Ok(0) = stream_reader.read_until(0x04, &mut recv) {
            println!("{} Disconnected!", &name);
            return Ok(());
        }
        let recv = String::from_utf8(recv.clone())
            .unwrap_or("Could not parse bytes".to_string());

        msg_chan.send(UserMessage::new(name.clone(), recv)).unwrap();
    }
}

fn echo_messages(msg_recv: Receiver<UserMessage>, users: &Arc<RwLock<Vec<Arc<TcpStream>>>>) {
    loop {
        let message = msg_recv.recv().unwrap();
        println!("message is {}", message.message);
        if let Err(e) = send_message(users, &message) {
            println!("Couldn't send message from {} error {}", message.user, e)
        }
    } 
}

fn send_message(users: &Arc<RwLock<Vec<Arc<TcpStream>>>>, message: &UserMessage) -> Result<(), Box<dyn Error>> {
    for user in &*users.write().unwrap() {
        let mut t: &TcpStream = user;
        t.write_all(format!("{}: {}\x04", message.user, message.message).as_bytes())?;
    }

    Ok(())
}
