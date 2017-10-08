#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate rand;
extern crate tui;
extern crate termion;

mod position;
mod world;
mod state;
mod actions;

use std::io;

use rand::Rng;

use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Paragraph, border};
use tui::style::Style;
use tui::layout::{Group, Direction, Size, Rect};

use state::State;
use world::World;
use actions::Actions;

fn main() {
    let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        #t...\n\
        .....\n\
        ";

    match World::build_from_str(source) {
        Err(msg) => {
            println!("Failed to build world: {}", msg);
            println!("Using source:");
            println!("{}", source);
        }
        Ok(w) => {
            match State::build_from_str(source, &w) {
                Err(msg) => {
                    println!("Failed to build state: {}", msg);
                    println!("Using state:");
                    println!("{}", source);
                }

                Ok(state) => {
                    let result = RandomSolver::new(state.clone(), 200);

                    if let Some(actions) = result.solution {
                        // println!(
                        //     "Found solution in {} steps, it is {} steps long.",
                        //     result.iterations,
                        //     actions.len()
                        // );

                        if let Err(error) = run_simulation(state, &actions) {
                            println!("IO error : {:?}", error);
                        }

                    // let mut display_state = state.clone();
                    // println!("-------------- {} -----------------", 0);
                    // show_state(&display_state);

                    // for (i, a) in actions.into_iter().enumerate() {
                    //     display_state = display_state.apply_action(a);
                    //     println!("-------------- {} -----------------", i + 1);
                    //     show_state(&display_state);
                    // }


                    } else {
                        println!("Failed to find solution in {} steps.", result.iterations);
                    }
                }
            }
        }
    }
}

fn run_simulation(state: State, actions: &[Actions]) -> Result<(), std::io::Error> {

    let mut simulation = Simulation::new(state, actions);

    let stdin = io::stdin();

    let backend = TermionBackend::new()?;
    let mut terminal = Terminal::new(backend)?;

    terminal.resize(simulation.term_size)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    simulation.draw(&mut terminal)?;


    for c in stdin.keys() {
        let evt = c?;

        match evt {
            event::Key::Esc => break,
            event::Key::Right => simulation.advance_step(1),
            event::Key::Left => simulation.advance_step(-1),
            _ => (),
        }

        simulation.draw(&mut terminal)?;
    }

    // Force a full re-draw so that it the cursor is at the end.
    terminal.resize(simulation.term_size)?;
    simulation.draw(&mut terminal)?;

    terminal.show_cursor()?;

    Ok(())
}


struct Simulation {
    states: Vec<String>,
    step: isize,
    max_step: isize,
    term_size: Rect,
    state_height: u16,
    state_width: u16,
}

impl Simulation {
    fn new(mut state: State, actions: &[Actions]) -> Simulation {

        let mut states = Vec::with_capacity(actions.len() + 1);

        states.push(show_state(&state));

        for a in actions {
            state = state.apply_action(*a);
            states.push(show_state(&state));
        }

        let state_height = (state.world.height + 2) as u16;
        let state_width = std::cmp::max(state.world.width + 2, 10) as u16;

        let term_width = std::cmp::max(state_width, 80);
        let term_height = state_height + 5;

        let term_size = Rect::new(0, 0, term_width, term_height);

        Simulation {
            states,
            step: 0,
            max_step: actions.len() as isize,
            term_size,
            state_height,
            state_width,
        }
    }

    fn advance_step(&mut self, increment: isize) {

        self.step += increment;

        if self.step < 0 {
            self.step = 0;
        } else if self.step > self.max_step {
            self.step = self.max_step;
        }
    }

    fn draw(&self, mut t: &mut Terminal<TermionBackend>) -> Result<(), std::io::Error> {

        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(self.state_height), Size::Percent(100)])
            .render(&mut t, &self.term_size, |mut t, chunks| {
                Group::default()
                    .direction(Direction::Horizontal)
                    .sizes(
                        &[Size::Fixed(10), Size::Fixed(self.state_width), Size::Min(0)],
                    )
                    .render(&mut t, &chunks[0], |mut t, chunks| {

                        let title = &format!("Step {:^3}", self.step);
                        let title_block = Block::default().title(title).borders(border::ALL);

                        let inner_rect = title_block.inner(&chunks[1]);
                        title_block.render(&mut t, &chunks[1]);

                        Group::default()
                            .direction(Direction::Horizontal)
                            .sizes(&[Size::Percent(15), Size::Percent(70), Size::Percent(15)])
                            .render(&mut t, &inner_rect, |mut t, chunks| {

                                Paragraph::default()
                                    .text(&self.states[self.step as usize])
                                    .render(&mut t, &chunks[1]);
                            });
                    });

                let instructions = format!(
                    "Left/Right arrow to advance.\n\
                     Escape to exit.\n\
                     \n\
                     Solved in {} steps.",
                    self.max_step
                );

                Paragraph::default().wrap(true).text(&instructions).render(
                    &mut t,
                    &chunks
                        [1],
                );
            });

        t.draw()?;

        Ok(())
    }
}

fn show_state(state: &State) -> String {
    match state.display() {
        Err(msg) => format!("Failed to display: {}", msg),
        Ok(state_str) => state_str,
    }
}

struct RandomSolver {
    iterations: u32,
    solution: Option<Vec<Actions>>,
}

impl RandomSolver {
    fn new(state: State, max_iterations: u32) -> RandomSolver {

        let mut rng = rand::thread_rng();

        let mut iterations = 0;
        let mut applied_actions = Vec::new();

        let mut current_state = state;

        loop {
            if current_state.at_destination() {
                break RandomSolver {
                    iterations,
                    solution: Some(applied_actions),
                };
            }

            if iterations >= max_iterations {
                break RandomSolver {
                    iterations,
                    solution: None,
                };
            }

            iterations += 1;

            let action: Actions = rng.gen();

            applied_actions.push(action);
            current_state = current_state.apply_action(action);
        }
    }
}
