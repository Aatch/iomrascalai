/************************************************************************
 *                                                                      *
 * Copyright 2014 Urban Hafner, Thomas Poinsot                          *
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

use board::{Board, Empty, White, Black};
use board::coord::Coord;

#[test]
fn getting_a_valid_coord_returns_a_color(){
  let b = Board::new(19, 6.5);

  assert_eq!(b.get(Coord::new(10,10)), Empty);
}

#[test]
#[should_fail]
fn getting_invalid_coordinates_fails() {
  let b = Board::new(19, 6.5);
  b.get(Coord::new(14, 21));
  b.get(Coord::new(21, 14));
}

#[test]
fn _19_19_is_a_valid_coordinate(){
  let b = Board::new(19, 6.5);

  assert_eq!(b.get(Coord::new(19, 19)), Empty);
}

#[test]
#[should_fail]
fn _0_0_is_not_a_valid_coordinate(){
  let b = Board::new(19, 6.5);

  b.get(Coord::new(0, 0));
}

#[test]
fn get_komi(){
  let b = Board::new(19, 6.5);

  assert_eq!(b.komi(), 6.5f32)
}

#[test]
fn play_adds_a_stone_to_the_correct_position() {
  let mut b = Board::new(19, 6.5); 

  b = b.play(White, 14, 14);
  
  assert!(b.get(Coord::new(14, 14)) == White);=

  for i in range(1u8, 20) {
    for j in range(1u8 , 20) {
      assert!(b.get(Coord::new(i, j)) == Empty || (i == 14 && j == 14));
    }
  }
}

#[test]
fn is_inside_valid_coords_pass() {
  let b = Board::new(19, 6.5);
  assert!(b.is_inside(Coord::new(1,1)));
  assert!(b.is_inside(Coord::new(19,19)));
  assert!(b.is_inside(Coord::new(10,10)));
}

#[test]
fn is_inside_0_0_fails() {
  let b = Board::new(19, 6.5);
  assert!(!b.is_inside(Coord::new(0,0)));
}

#[test]
fn is_inside_invalid_coords_fail() {
  let b = Board::new(19, 6.5);
  assert!(!b.is_inside(Coord::new(4,21)));
  assert!(!b.is_inside(Coord::new(21,4)));

  let c = Board::new(9, 6.5);
  assert!(!c.is_inside(Coord::new(18,18)));
}

#[test]
fn two_way_merging_works() {
  let mut b = Board::new(19, 6.5);

  b = b.play(White, 10, 10);
  b = b.play(White, 10, 12);

  assert_eq!(b.chains.len(), 3);

  b = b.play(White, 10, 11);
  let c_id = b.get_chain(Coord::new(10, 10)).id;

  assert_eq!(b.get_chain(Coord::new(10, 11)).id, c_id);
  assert_eq!(b.get_chain(Coord::new(10, 12)).id, c_id);
  assert_eq!(b.chains.len(), 2)
}

#[test]
fn three_way_merging_works() {
  let mut b = Board::new(19, 6.5);

  b = b.play(White, 10, 10);
  b = b.play(White, 11, 11);
  b = b.play(White, 10, 12);

  assert_eq!(b.chains.len(), 4);

  b = b.play(White, 10, 11);
  let c_id = b.get_chain(Coord::new(10, 10)).id;

  assert_eq!(b.get_chain(Coord::new(10, 11)).id, c_id);
  assert_eq!(b.get_chain(Coord::new(11, 11)).id, c_id);
  assert_eq!(b.get_chain(Coord::new(10, 12)).id, c_id);
  assert_eq!(b.chains.len(), 2)
}

#[test]
fn four_way_merging_works() {
  let mut b = Board::new(19, 6.5);

  b = b.play(White, 10, 10);
  b = b.play(White,  9, 11);
  b = b.play(White, 11, 11);
  b = b.play(White, 10, 12);

  assert_eq!(b.chains.len(), 5);

  b = b.play(White, 10, 11);
  let c_id = b.get_chain(Coord::new(10, 10)).id;

  assert_eq!(b.get_chain(Coord::new(10, 11)).id, c_id);
  assert_eq!(b.get_chain(Coord::new(9 , 11)).id, c_id);
  assert_eq!(b.get_chain(Coord::new(11, 11)).id, c_id);
  assert_eq!(b.get_chain(Coord::new(10, 12)).id, c_id);
  assert_eq!(b.chains.len(), 2)
}

#[test]
fn playing_on_all_libs_in_corner_should_capture() {
  let mut b = Board::new(19, 6.5);

  b = b.play(Black, 1, 1);
  b = b.play(White, 1, 2);
  b = b.play(White, 2, 1);

  assert_eq!(b.get(Coord::new(1, 1)), Empty);
}

#[test]
fn playing_on_all_libs_on_side_should_capture() {
  let mut b = Board::new(19, 6.5);

  b = b.play(Black, 1, 3);
  b = b.play(White, 1, 2);
  b = b.play(White, 1, 4);
  b = b.play(White, 2, 3);

  assert_eq!(b.get(Coord::new(1, 3)), Empty);
}

#[test]
fn playing_on_all_libs_should_capture() {
  let mut b = Board::new(19, 6.5);

  b = b.play(Black, 4, 4);
  b = b.play(White, 4, 3);
  b = b.play(White, 4, 5);
  b = b.play(White, 3, 4);
  b = b.play(White, 5, 4);

  assert_eq!(b.get(Coord::new(4, 4)), Empty); 
}

#[test]
fn playing_on_all_libs_of_a_chain_should_capture() {
  let mut b = Board::new(19, 6.5);

  b = b.play(Black, 4, 4);
  b = b.play(Black, 4, 5);

  b = b.play(White, 4, 3);
  b = b.play(White, 3, 4);
  b = b.play(White, 5, 4);
  b = b.play(White, 3, 5);
  b = b.play(White, 5, 5);
  b = b.play(White, 4, 6);

  assert_eq!(b.get(Coord::new(4, 4)), Empty); 
  assert_eq!(b.get(Coord::new(4, 5)), Empty); 
}

#[test]
fn playing_on_all_libs_of_a_bent_chain_should_capture() {
  let mut b = Board::new(19, 6.5);

  b = b.play(Black, 4, 4);
  b = b.play(Black, 4, 5);
  b = b.play(Black, 3, 4);

  b = b.play(White, 3, 3);
  b = b.play(White, 4, 3);
  b = b.play(White, 2, 4);
  b = b.play(White, 5, 4);
  b = b.play(White, 3, 5);
  b = b.play(White, 5, 5);
  b = b.play(White, 4, 6);

  assert_eq!(b.get(Coord::new(4, 4)), Empty); 
  assert_eq!(b.get(Coord::new(4, 5)), Empty); 
  assert_eq!(b.get(Coord::new(3, 4)), Empty); 
}



