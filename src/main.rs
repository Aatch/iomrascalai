/************************************************************************
 *                                                                      *
 * Copyright 2014 Urban Hafner, Thomas Poinsot                          *
 * Copyright 2015 Urban Hafner, Thomas Poinsot, Igor Polyakov           *
 *                                                                      *
 * This file is part of Iomrascálaí.                                    *
 *                                                                      *
 * Iomrascálaí is free software: you can redistribute it and/or modify  *
 * it under the terms of the GNU General Public License as published by *
 * the Free Software Foundation, either version 3 of the License, or    *
 * (at your option) any later version.                                  *
 *                                                                      *
 * Iomrascálaí is distributed in the hope that it will be useful,       *
 * but WITHOUT ANY WARRANTY; without even the implied warranty of       *
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the        *
 * GNU General Public License for more details.                         *
 *                                                                      *
 * You should have received a copy of the GNU General Public License    *
 * along with Iomrascálaí.  If not, see <http://www.gnu.org/licenses/>. *
 *                                                                      *
 ************************************************************************/
#![feature(core)]
#![feature(mpsc_select)]
#![feature(owned_ascii_ext)]
#![feature(plugin)]
#![feature(scoped)]
#![feature(slice_extras)]
#![feature(test)]
#![feature(vec_push_all)]
#![plugin(regex_macros)]
extern crate core;
#[macro_use] extern crate enum_primitive;
extern crate getopts;
extern crate num;
extern crate quicksort;
extern crate rand;
extern crate regex;
#[no_link] extern crate regex_macros;
extern crate smallvec;
extern crate test;
extern crate time;
#[macro_use(strenum)] extern crate strenum;

use config::Config;
use gtp::driver::Driver;

use getopts::Options;
use std::env::args;
use std::io::Write;
use std::process::exit;

macro_rules! log(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

mod board;
mod config;
mod engine;
mod game;
mod gtp;
mod playout;
mod ruleset;
mod score;
mod sgf;
mod timer;
mod version;

pub fn main() {
    let mut config = Config::default();
    let mut opts = Options::new();
    let args : Vec<String> = args().collect();

    opts.optopt("e", "engine", "select an engine (defaults to uct)", "amaf|mc|random|uct");
    opts.optopt("p", "playout", "type of playout to use (defaults to no-self-atari)", "light|no-self-atari");

    config.setup(&mut opts);

    let matches = match opts.parse(args.tail()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            exit(1);
        }
    };

    match config.set_from_opts(&matches, &opts, &args) {
        Ok(opt) => {
            match opt {
                Some(s) => {
                    println!("{}", s);
                    exit(0);
                }
                None => {}
            }
        },
        Err(s) => {
            println!("{}", s);
            exit(1);
        }
    }

    let playout = playout::factory(matches.opt_str("p"), config);

    let engine = engine::factory(matches.opt_str("e"), config, playout);

    log!("Current configuration: {:?}", config);

    Driver::new(config, engine);
}
