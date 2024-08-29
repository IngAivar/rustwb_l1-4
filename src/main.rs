use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::env;

fn main() {
    // Получаем количество воркеров из аргументов командной строки.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number_of_workers>", args[0]);
        std::process::exit(1);
    }

    let num_workers: usize = args[1].parse().expect("Please provide a valid number");

    // Создаем канал для отправки данных от главного потока к воркерам.
    let (tx, rx) = mpsc::channel();

    // Оборачиваем приемник в Arc<Mutex<Receiver<T>>> для безопасного использования в потоках.
    let rx = Arc::new(Mutex::new(rx));

    // Запускаем поток, который будет записывать данные в канал.
    thread::spawn(move || {
        let mut count = 0;
        loop {
            let message = format!("Message {}", count);
            println!("Main thread sending: {}", &message);
            if tx.send(message).is_err() {
                println!("Main thread: receiver dropped, exiting.");
                break;
            }
            count += 1;
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Создаем и запускаем воркеры.
    let mut handles = vec![];
    for id in 0..num_workers {
        let rx = Arc::clone(&rx);
        let handle = thread::spawn(move || {
            loop {
                let message = {
                    // Блокируем мьютекс и получаем доступ к приемнику.
                    let rx_lock = rx.lock().unwrap();
                    rx_lock.recv()
                };

                match message {
                    Ok(msg) => println!("Worker {} received: {}", id, msg),
                    Err(_) => {
                        println!("Worker {}: channel closed, exiting.", id);
                        break; // Канал закрыт.
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Ожидаем завершения всех воркеров.
    for handle in handles {
        handle.join().expect("Worker thread panicked");
    }
}

// прописать в консоли cargo run -- 5