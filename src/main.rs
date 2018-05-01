#[macro_use] extern crate serde_derive;
#[macro_use] extern crate hyper;

extern crate time;
extern crate futures;
extern crate gotham;
#[macro_use] extern crate gotham_derive;
extern crate mime;
extern crate serde;
extern crate serde_json;

extern crate clap;

extern crate homepage_data;
extern crate homepage_view;

mod routes;

use clap::{Arg, App};

pub fn main() {
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
