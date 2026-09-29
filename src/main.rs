use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;

fn main() -> io::Result<()> {
    let port = 3030;
    let addr = format!("0.0.0.0:{}", port);

    // 1. Create socket, bind to address, and listen for connections
    let listener = TcpListener::bind(&addr)?;
    println!("Listening on {}", addr);

    listener.set_nonblocking(true)?;
    let epoll_fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    if epoll_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let listener_raw_fd = listener.as_raw_fd();
    let mut listen_event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: listener_raw_fd as u64,
    };

    let res = unsafe {
        libc::epoll_ctl(
            epoll_fd,
            libc::EPOLL_CTL_ADD,
            listener_raw_fd,
            &mut listen_event,
        )
    };
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut clients: HashMap<i32, TcpStream> = HashMap::new();

    let mut events = [libc::epoll_event { events: 0, u64: 0 }; 1024];

    loop {
        let n_events =
            unsafe { libc::epoll_wait(epoll_fd, events.as_mut_ptr(), events.len() as i32, -1) };

        if n_events < 0 {
            return Err(io::Error::last_os_error());
        }

        for i in 0..n_events as usize {
            let event = events[i];
            let ready_fd = event.u64 as i32;

            if ready_fd == listener_raw_fd {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("New connection from {}", addr);
                        stream.set_nonblocking(true)?;
                        let client_fd = stream.as_raw_fd();

                        let mut client_event = libc::epoll_event {
                            events: libc::EPOLLIN as u32,
                            u64: client_fd as u64,
                        };

                        unsafe {
                            libc::epoll_ctl(
                                epoll_fd,
                                libc::EPOLL_CTL_ADD,
                                client_fd,
                                &mut client_event,
                            );
                        }

                        clients.insert(client_fd, stream);
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            } else {
                let mut disconnected = false;

                if let Some(stream) = clients.get_mut(&ready_fd) {
                    let mut buffer = [0u8; 1024];

                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            // Client closed connection (EOF)
                            println!("Client {} disconnected", ready_fd);
                            disconnected = true;
                        }
                        Ok(n) => {
                            if let Err(e) = stream.write_all(&buffer[..n]) {
                                eprintln!("Write error to client {}: {}", ready_fd, e);
                                disconnected = true;
                            }
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // Nothing left to read
                        }
                        Err(e) => {
                            eprintln!("Read error on client {}: {}", ready_fd, e);
                            disconnected = true;
                        }
                    }
                }

                if disconnected {
                    unsafe {
                        libc::epoll_ctl(
                            epoll_fd,
                            libc::EPOLL_CTL_DEL,
                            ready_fd,
                            std::ptr::null_mut(),
                        );
                    }
                    clients.remove(&ready_fd); // Drops TcpStream -> kernel closes fd
                }
            }
        }
    }

    Ok(())
}
