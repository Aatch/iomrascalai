/************************************************************************
 *                                                                      *
 * Copyright 2015 Urban Hafner                                          *
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

#![cfg(test)]

use config::Config;
use playout::Playout;
use playout;

fn config() -> Config {
    Config::default()
}

fn playout() -> Box<Playout> {
    playout::factory(None, config())
}

#[test]
fn factory_returns_uct_by_default() {
    let engine = super::factory(None, config(), playout());
    assert_eq!("uct", engine.engine_type());
}

#[test]
fn factory_returns_random_engine_when_give_random() {
    let engine = super::factory(Some(String::from("random")), config(), playout());
    assert_eq!("random", engine.engine_type());
}

#[test]
fn factory_returns_simple_mc_when_given_mc() {
    let engine = super::factory(Some(String::from("mc")), config(), playout());
    assert_eq!("simple-mc", engine.engine_type());
}

#[test]
fn factory_returns_amaf_when_given_amaf() {
    let engine = super::factory(Some(String::from("amaf")), config(), playout());
    assert_eq!("amaf", engine.engine_type());
}

#[test]
fn factory_returns_uct_for_any_other_string() {
    let engine = super::factory(Some(String::from("foo")), config(), playout());
    assert_eq!("uct", engine.engine_type());
}
