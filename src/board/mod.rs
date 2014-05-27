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
use std::vec::Vec;
use board::chain::Chain;
use board::coord::Coord;

mod board_test;
mod coord_test;
mod chain_test;

mod coord;
mod chain;

#[deriving(Clone, Show, Eq)]
pub enum Color {
    White,
    Black,
    Empty
}

impl Color {
    fn opposite(&self) -> Color {
        match *self {
            White => Black,
            Black => White,
            Empty => Empty
        }
    }
}

#[deriving(Clone)]
pub struct Board {
    komi: f32,
    size: u8,
    board: Vec<uint>,
    chains: Vec<Chain>
}

impl Board {
    pub fn new(size: uint, komi: f32) -> Board {
        Board {
            komi: komi,
            size: size as u8,
            board: Vec::from_fn(size*size, |_| 0),
            chains: vec!(Chain::new(0, Empty))
        }
    }

    // Note: This method uses 1-1 as the origin point, not 0-0. 19-19 is a valid coordinate in a 19-sized board, while 0-0 is not.
    //       this is done because I think it makes more sense in the context of go. (Least surprise principle, etc...)
    pub fn get(&self, c: Coord) -> Color {
        if self.is_inside(c) {
            self.get_chain(c).color
        } else {
            fail!("You have requested a stone outside of the board");
        }
    }

    pub fn get_chain<'a>(&'a self, c: Coord) -> &'a Chain {
        if self.is_inside(c) {
            let chain_id = *self.board.get(c.to_index(self.size));
            self.chains.get(chain_id)
        } else {
            fail!("You have requested a chain outside of the board");
        }
    }

    pub fn komi(&self) -> f32 {
        self.komi
    }

    fn is_inside(&self, c: Coord) -> bool {
        1 <= c.col && c.col <= self.size && 1 <= c.row && c.row <= self.size
    }

    // Note: Same as get(), the board is indexed starting at 1-1
    pub fn play(&self, color: Color, col: u8, row: u8) -> Board {
        let new_coords      = Coord::new(col, row);

        // We check the validity of the coords.
        let mut new_board = if self.is_inside(new_coords) {
            self.clone()
        } else {
            fail!("The coordinate you have entered ({} {}) are invalid", col, row);
        };

        let friend_neigh_chains_id = self.find_neighbouring_friendly_chains_ids(new_coords, color);

        /*
         * If there is 0 friendly neighbouring chain, we create one, and assign the coord played to that new chain.
         * If there is 1, we assign the stone to that chain.
         * If there are more, we assign the stone to one chain, then merge the others into that chain, then remove the old chains from
         * board.chains, then we lower by 1 the ids of all stones with chain ids higher than the removed chains,
         * and finally we reassign the correct chain_id to each stone in the final chain.
        */
        match friend_neigh_chains_id.len() {
            0 => new_board.create_new_chain(color, new_coords),
            1 => {
                let final_chain_id = *friend_neigh_chains_id.get(0);
                new_board.add_coord_to_chain(new_coords, final_chain_id);
            },
            _ => {
                // Note: We know that friend_neigh_chains_id is sorted, so whatever chains we remove, 
                // we know that the id of the final_chain is still valid.
                let final_chain_id        = *friend_neigh_chains_id.get(0);
                let mut nb_removed_chains = 0;

                // We assign the stone to the final chain
                new_board.add_coord_to_chain(new_coords, final_chain_id);
                
                for &other_chain_old_id in friend_neigh_chains_id.slice(1, friend_neigh_chains_id.len()).iter() {
                    // The ids stored in friend_neigh_chains_id may be out of date since we remove chains from new_board.chains
                    // These id is the correct one at this step of the removals
                    let other_chain_id = other_chain_old_id - nb_removed_chains;  

                    // We merge the other chain into the final chain.
                    let other_chain = new_board.chains.get(other_chain_id).clone();
                    new_board.chains.get_mut(final_chain_id).merge(&other_chain);

                    // We remove the old chain.
                    new_board.chains.remove(other_chain_id);

                    // We update the ids inside the chains
                    new_board.update_chains_ids_after_removed_chain(other_chain_id);
                    
                    nb_removed_chains += 1;
                }
                new_board.update_board_ids();
            }
        }

        // Then we loop up the enemy chains neighours of the new stone, and we decrease their libs by one
        new_board.update_enemy_chains_libs(new_coords, color.opposite());

        new_board.remove_chains_with_no_libs();

        new_board.update_chains_ids();
        new_board.update_board_ids();

        new_board
    }

    fn count_libs(&self, c: Coord) -> uint {
        c.neighbours().iter().filter(|&c| self.is_inside(*c) && self.get(*c) == Empty).len()
    }

    fn update_chains_ids_after_removed_chain(&mut self, removed_chain_id: uint) {
        // We decrease by one every index in chains that is higher than other_chain_id
        for chain in self.chains.mut_iter() {
            if chain.id > removed_chain_id {chain.id -= 1;}
        }
    }

    fn update_board_ids(&mut self) {
        for chain in self.chains.clone().iter() {
            for &coord in chain.coords().iter() {
                *self.board.get_mut(coord.to_index(self.size)) = chain.id;
            }
        }
    }

    fn find_neighbouring_friendly_chains_ids(&self, c: Coord, color: Color) -> Vec<uint> {
        let mut friend_neigh_chains_id: Vec<uint> = c.neighbours()
                  .iter()
                  .filter(|&c| self.is_inside(*c) && self.get(*c) == color)
                  .map(|&c| self.get_chain(c).id)
                  .collect();

        // We need to sort the chain by ascending id so that later we know that friend_neigh_chains_id[0] has the lowest id.
        // It also helps with keeping track of the ids of the chain yet-to-merge as their ids will always decrease by nb of chains
        // merged before them.
        friend_neigh_chains_id.sort();
        friend_neigh_chains_id.dedup();
        friend_neigh_chains_id
    }

    fn update_enemy_chains_libs(&mut self, coord: Coord, adv_color: Color) {
        if self.get_chain(coord).libs == 0 {
            fail!("You can't play a suicide move");
        }

        let mut adv_chains_ids: Vec<uint> = coord.neighbours()
                  .iter()
                  .filter(|&c| self.is_inside(*c) && self.get(*c) == adv_color)
                  .map(|&c| self.get_chain(c).id)
                  .collect();

        adv_chains_ids.sort();
        adv_chains_ids.dedup();

        for &id in adv_chains_ids.iter() {
            self.chains.get_mut(id).libs -= 1;
        }
    }

    fn update_chains_ids(&mut self) {
        for i in range(1, self.chains.len()) {
            self.chains.get_mut(i).id = i;
        }
    }

    fn remove_chains_with_no_libs(&mut self) {
        // First we remove the stones contained by the chain.
        let coords_to_remove = self.chains.iter()
                                          .filter(|chain| chain.libs == 0)
                                          .fold(Vec::<Coord>::new(), |acc, chain| acc.append(chain.coords().as_slice()));

        for &coord in coords_to_remove.iter() {
            self.remove_stone(coord);
        }

        // Then we remove the chain from the board.chains Vec
        let mut ids_to_remove: Vec<uint> = self.chains.iter()
                                                  .filter(|chain| chain.libs == 0)
                                                  .map(|chain| chain.id)
                                                  .collect();

        // The sorting is needed to make sure we remove the chain in the right order:
        // if it wasn't sorted, then we might remove a later chain before an earlier one
        // which would not impact the early one's id. Then, id-nb_removed would not point
        // to the correct id.
        ids_to_remove.sort();

        let mut nb_removed = 0;
        for id in ids_to_remove.iter() {
            self.chains.remove(id - nb_removed);
            nb_removed += 1;
        }
    }

    fn create_new_chain(&mut self, color: Color, init_coord: Coord) {
        let new_chain_id    = self.chains.len();
        let mut new_chain   = Chain::new(new_chain_id, color);
        new_chain.add_stone(init_coord, self.count_libs(init_coord));
        self.chains.push(new_chain);
        *self.board.get_mut(init_coord.to_index(self.size)) = new_chain_id;
    }

    fn add_coord_to_chain(&mut self, coord: Coord, chain_id: uint) {
        let coord_libs = self.count_libs(coord);
        self.chains.get_mut(chain_id).add_stone(coord, coord_libs);
       *self.board.get_mut(coord.to_index(self.size)) = chain_id;    
    }

    fn remove_stone(&mut self, c: Coord) {
        *self.board.get_mut(c.to_index(self.size)) = 0;
    }

    pub fn show(&self) {
        println!("komi: {}", self.komi());

        // First we print the board
        for row in range(1u8, self.size+1).rev() {

            // Prints the row number
            print!("{:2} ", row);

            // Prints the actual row
            for col in range(1u8, self.size+1) {
                let current_coords = Coord::new(col, row);

                if self.get(current_coords) == Empty {
                    let hoshis = &[4u8,10,16];
                    if   hoshis.contains(&row) && hoshis.contains(&col) {print!("+ ")}
                    else                                                {print!(". ")}
                } else if self.get(current_coords) == White {print!("O ")}
                  else if self.get(current_coords) == Black {print!("X ")}
            }
            println!("");
        }

        // Then we print the col numbers under the board
        print!("{:3}", "");
        for col in range(1, self.size+1) {
            print!("{:<2}", col);
        }

        println!("");
    }

    pub fn show_chains(&self) {
        for c in self.chains.iter() {
            println!("{}", c.show());
        }
    }
}
