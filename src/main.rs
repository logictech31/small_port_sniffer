use std::{env, 
    process, 
    thread, 
    sync::mpsc::{Sender, channel}, 
    net::{IpAddr, TcpStream}, 
    str::FromStr,
    io::{self, Write},
};

struct Arguments {
    flag: String,
    ip: IpAddr, // this is an enum IP4 and IP6
    threads: u16,
}

const MAX: u16 = 65535;

impl Arguments {
    // We will create method new
    // will take reference to a vector String
    // And will return a "Result" that will have our arguments struct in it's Ok() portion
    // or it will have a slice of static string inside of the Err() portion
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments!");
        } else if args.len() > 4 {
            return Err("Too many arguments!");
        }
        // We will clone the Ip Address here
        let f = args[1].clone();
        // We destruct our IP Address from the string which IF returns an Ok() will exec the nest
        // then we will use the IpAddr
        if let Ok(ip) = IpAddr::from_str(&f) {
            return Ok(Arguments {flag: String::from(""), ip, threads: 4});
        } else {
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("-help") && args.len() == 2 {
                println!("Usage -j to select how many threads you want 
                \r\n        -h or -help to show this help message");
                return Err("Help.");
            } else if flag.contains("-h") || flag.contains("-help") {
                return Err("Too many arguments");
            } else if flag.contains("-j") {
                let ip = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("Not a valid IpAddr; must be IPv4 or IPv6")
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("Failed to pass thread number")
                };
                return Ok(Arguments {threads, flag, ip});
            } else {
                return Err("Invalid syntax");
            }
        }
    }
}

// We will declare the scan function here
// Our thread N will look at port N
fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;

    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            },
            Err(_) => {}
        }

        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    // We will collect the arguments from the user
    let args: Vec<String> = env::args().collect();
    let prog = args[0].clone();
    // The Unwrap or else takes a closure
    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("Help.") {
                process::exit(0);
            } else {
                eprintln!("{} problem parsing arguments: {}", prog, err);
                process::exit(0);
            }
        }
    );
    
    // we bind the number our arguments.thread to variable
    let num_threads = arguments.threads;
    let addr = arguments.ip;
    // We then instantiate a channel where we get the transmitter and receiver
    let (tx, rx) = channel();
    for i in 0..num_threads {
        // We bind tx with a variable tx, enabling each thread to have a transmitter
        let tx = tx.clone();
        
        // In this move closure we will have a scan call
        // This scan call will pass the transmitter, the thread number, ip address, and the number of threads
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();

    for v in out {
        println!("{} is open", v);
    }
}
