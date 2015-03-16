/************************************************************************
 *                                                                      *
 * Copyright 2014 Urban Hafner                                          *
 * Copyright 2015 Urban Hafner, Thomas Poinsot                          *
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

pub use self::amaf::AmafMcEngine;
pub use self::simple::SimpleMcEngine;
pub use super::Engine;
pub use super::MoveStats;
use board::Board;
use board::Color;
use board::Move;
use board::Pass;
use board::Resign;
use game::Game;
use playout::Playout;

use rand::random;
use std::marker::MarkerTrait;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;

mod amaf;
mod simple;

pub trait McEngine: MarkerTrait {

    fn record_playout(&mut MoveStats, &Playout, bool);

}

fn gen_move<T: McEngine>(threads: usize, color: Color, game: &Game, sender: Sender<Move>, receiver: Receiver<()>) {
    let moves = game.legal_moves_without_eyes();
    if moves.is_empty() {
        log!("No moves to simulate!");
        sender.send(Pass(color));
        return;
    }
    let mut stats = MoveStats::new(&moves, color);
    let mut counter = 0;
    let (send_result, receive_result) = channel::<(MoveStats, usize)>();
    let (guards, halt_senders) = spin_up::<T>(color, threads, &moves, game, send_result);
    loop {
        select!(
            result = receive_result.recv() => {
                let (ms, count) = result.unwrap();
                stats.merge(&ms);
                counter += count;
            },
            _ = receiver.recv() => {
                log!("{} simulations", counter);
                finish(color, game, stats, sender, halt_senders);
                break;
            }
            )
    }
}

fn finish(color: Color, game: &Game, stats: MoveStats, sender: Sender<Move>, halt_senders: Vec<Sender<()>>) {
    if stats.all_losses() {
        log!("All simulations were losses");
        if game.winner() == color {
            sender.send(Pass(color));
        } else {
            sender.send(Resign(color));
        }
    } else {
        let (m, s) = stats.best();
        log!("Returning the best move ({}% wins)", s.win_ratio()*100.0);
        sender.send(m);
    }
    for halt_sender in halt_senders.iter() {
        halt_sender.send(());
    }
}

fn spin_up<'a, T: McEngine>(color: Color, threads: usize, moves: &'a Vec<Move>, game: &Game, send_result: Sender<(MoveStats<'a>, usize)>) -> (Vec<thread::JoinGuard<'a, ()>>, Vec<Sender<()>>) {
    let mut guards = Vec::new();
    let mut halt_senders = Vec::new();
    for _ in range(0, threads) {
        let (send_halt, receive_halt) = channel::<()>();
        halt_senders.push(send_halt);
        let send_result = send_result.clone();
        let guard = spin_up_worker::<T>(color, receive_halt, moves, game.board(), send_result);
        guards.push(guard);
    }
    (guards, halt_senders)
}

fn spin_up_worker<'a, T: McEngine>(color: Color, recv_halt: Receiver<()>, moves: &'a Vec<Move>, board: Board, send_result: Sender<(MoveStats<'a>, usize)>) -> thread::JoinGuard<'a, ()> {
    thread::scoped(move || {
        let runs = 100;
        let mut stats = MoveStats::new(moves, color);
        loop {
            for _ in range(0, runs) {
                let m = moves[random::<usize>() % moves.len()];
                let playout = Playout::run(&board, &m);
                let winner = playout.winner();
                T::record_playout(&mut stats, &playout, winner == color);
            }
            if recv_halt.try_recv().is_ok() {
                break;
            } else {
                send_result.send((stats, runs));
                stats = MoveStats::new(moves, color);
            }
        }
    })
}
