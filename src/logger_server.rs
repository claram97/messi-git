use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

pub fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let log = OpenOptions::new().append(true).open("src/log.log")?;
    let logfile = Arc::new(Mutex::new(log));

    let mut handles = vec![];
    while let Ok((mut client_stream, _socket_addr)) = listener.accept() {
        let logfile_shared = logfile.clone();
        let handle = thread::spawn(move || handle_client(&mut client_stream, logfile_shared));
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    Ok(())
}

fn handle_client(stream: &mut dyn Read, log: Arc<Mutex<File>>) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        if let (Ok(buf), Ok(mut file)) = (line, log.lock()) {
            file.write(buf.as_bytes())?;
            file.write("\n".as_bytes())?;
        }
    }
    Ok(())
}
