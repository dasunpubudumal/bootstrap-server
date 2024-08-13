use core::panic;
use std::{io::{self, prelude::*, BufReader}, net::{TcpListener, TcpStream}, vec};
use log::info;
use terminal_banner::Banner;
use std::io::ErrorKind;
use rand::prelude::*;

struct Neighbour {
    ip: String,
    port: String,
    username: String
}

fn main() {
    // Establishes a TCP connection. This should be communicated using Telnet (Telnet doesn't support UDP)
    // Use nc -n -v (ip) (port) < <(echo "<message>") to communicate

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut neighbour_vector: Vec<Neighbour> = vec![];

    let banner = Banner::new()
    .text("Bootstrap server".into())
    .text("Bootstrap server is listening to incoming connections.".into())
    .render();
    println!("{}", banner);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        let _ = handle_connection(stream, &mut neighbour_vector);
    }
}

fn handle_connection<'a>(mut stream: TcpStream, vector: &'a mut Vec<Neighbour>) -> std::io::Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let first = http_request.first().unwrap();
    let result = match process_command(&first, vector) {
        Ok(result) => {result},
        Err(err) => {panic!("{:?}", err)}
    };
    stream.write_all(result.as_bytes())
}

// Command format: LENGTH COMMAND ARGS...
/// Process incoming register requests from peer nodes
fn process_command<'a>(command_string: &'a str, nodes: &mut Vec<Neighbour>) -> Result<String, io::Error> {
    let mut command_args: std::iter::Peekable<std::str::Split<&str>> = command_string.split(" ").peekable();
    
    let length = command_args.next().unwrap();
    info!("Length: {:?}", length);

    return match command_args.next() {
        Some(command) => {
            if command == "REG" {
                let mut reply = String::from("REGOK");
                let ip = command_args.next().unwrap();
                let port = command_args.next().unwrap();
                let username = command_args.next().unwrap();

                if nodes.is_empty() {
                    reply = format!("{}{}", reply, "0");
                    nodes.push(Neighbour { ip: ip.to_string(), port: port.to_string(), username: username.to_string() });
                } else {
                    let mut is_okay: bool = true;
                    for node in nodes.iter() {
                        if node.port == port {
                            if node.username == username {
                                reply = format!("{}{}", reply, "9998");
                            } else {
                                reply = format!("{}{}", reply, "9997");
                            }
                            is_okay = false;
                        }
                    }
                    if is_okay {
                        if nodes.len() == 1 {
                            reply = format!("{} {} {}", "1", nodes.get(0).unwrap().ip, nodes.get(0).unwrap().port);
                        } else if nodes.len() == 2 {
                            reply = format!("{} {} {} {} {}", "2", nodes.get(0).unwrap().ip, nodes.get(0).unwrap().port, nodes.get(1).unwrap().ip, nodes.get(1).unwrap().port);
                        } else {
                            let mut rng = thread_rng();
                            let low: usize = 0;
                            let high = nodes.len();
                            let random_1 = rng.gen_range(low..high);
                            let mut random_2 = rng.gen_range(low..high);
                            while random_1 == random_2 {
                                random_2 = rng.gen_range(low..high);
                            }
                            println!("{} {}", &random_1, &random_2);
                            reply = format!("{} {} {} {} {}", "2", nodes.get(random_1).unwrap().ip, nodes.get(random_1).unwrap().port, nodes.get(random_2).unwrap().ip, nodes.get(random_2).unwrap().port);
                        }
                        nodes.push(Neighbour { ip: ip.to_string(), port: port.to_string(), username: username.to_string() })
                    }
                }
                return Ok(format!("{:04} {}", reply.len(), reply));
            } else if command == "UNREG" {
                let port = command_args.next().unwrap();
                let node_size = nodes.len();
                for idx in 0..node_size {
                    if nodes.get(idx).unwrap().port == port {
                        nodes.remove(idx);
                        return Ok(String::from("0012 UNROK 0"));
                    }
                }
            } else if command == "ECHO" {
                for node in nodes.iter() {
                    print!("{:?} {:?} {:?}", node.ip, node.port, node.username);
                }
                return Ok(String::from("0012 ECHOK 0"));
            }
            Ok(String::from(""))
        },
        None => return Err(io::Error::new(ErrorKind::Other, "Invalid length!"))
    }
}

#[cfg(test)]
mod tests{

    use super::*;

    #[test]
    fn test_process_command() {

    }

}
