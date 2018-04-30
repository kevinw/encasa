use gotham;
use routes;
use clap::{Arg, App};

pub fn run_command_line() {
    let matches = App::new("homepage")
                          .version("0.1")
                          .author("Kevin W. <kevinwatters@gmail.com>")
                          .about("A better start page")
                          .arg(Arg::with_name("port")
                               .short("port")
                               .long("port")
                               .value_name("PORT")
                               .help("Sets a custom port to use")
                               .takes_value(true))
                          .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let port_str = matches.value_of("port").unwrap_or("7878");
    println!("Value for config: {}", port_str);

    let addr = format!("127.0.0.1:{}", port_str);
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, routes::router());
}
