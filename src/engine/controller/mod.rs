/************************************************************************
 *                                                                      *
 * Copyright 2015 Urban Hafner, Igor Polyakov                           *
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

use board::Color;
use board::Move;
use config::Config;
use engine::Engine;
use game::Game;
use timer::Timer;
use std::io::Write;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep_ms;

mod test;

pub struct EngineController<'a> {
    config: Config,
    engine: Box<Engine + 'a>,
}

impl<'a> EngineController<'a> {

    pub fn new<'b>(config: Config, engine: Box<Engine + 'b>) -> EngineController<'b> {
        EngineController {
            config: config,
            engine: engine,
        }
    }

    pub fn reset(&mut self) {
        self.engine.reset();
    }

    pub fn run_and_return_move(&mut self, color: Color, game: &Game, timer: &Timer, send_move: Sender<Move>) {
        let budget = self.budget(timer, game);
        let (send_move_to_controller, receive_move_from_engine) = channel();
        let (send_signal_to_engine, receive_signal_from_controller) = channel::<()>();
        // Saving the guard into a variable is necessary. Otherwise
        // the code blocks right here.
        let _guard = thread::scoped(|| {
            self.engine.gen_move(color, game, send_move_to_controller, receive_signal_from_controller);
        });
        let (send_time_up_to_controller, receive_time_up) = channel();
        thread::spawn(move || {
            sleep_ms(budget);
            send_time_up_to_controller.send(()).unwrap();
        });
        select!(
            r = receive_move_from_engine.recv() => {
                send_move.send(r.unwrap()).unwrap();
            },
            _ = receive_time_up.recv() => {
                send_signal_to_engine.send(()).unwrap();
                let m = receive_move_from_engine.recv().unwrap();
                send_move.send(m).unwrap();
            }
        )
    }

    fn budget(&self, timer: &Timer, game: &Game) -> u32 {
        let budget = timer.budget(game);
        if self.config.log {
            log!("Thinking for {}ms ({}ms time left)", budget, timer.main_time_left());
        }
        budget
    }

}
