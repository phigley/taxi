
use std::io;

use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Paragraph};
use tui::layout::{Group, Direction, Size, Rect};

use taxi::state::State;
use taxi::actions::Actions;

pub struct Replay {
    states: Vec<String>,
    actions: Vec<Actions>,
    step_height: u16,
    summary_height: u16,
    summary: String,
    max_step: isize,
    term_size: Rect,
    state_height: u16,
}

impl Replay {
    pub fn new(mut state: State, actions: &[Actions]) -> Replay {

        let mut states = Vec::with_capacity(actions.len() + 1);

        states.push(state.display());

        for a in actions {
            state = state.apply_action(*a);
            states.push(state.display());
        }

        let state_height = (2 * state.world.height + 1) as u16;

        let num_actions = actions.len();

        let summary =
            format!(
                    "\n\
                     Solved in {} steps.\n\
                     \n\
                     Left/Right arrow to advance.\n\
                     Escape to exit.\n\
                     ",
                    num_actions,
                );

        let summary_height = 5;

        let step_height = 3;

        let term_width = 80;
        let term_height = state_height + summary_height + step_height;



        let term_size = Rect::new(0, 0, term_width, term_height);

        Replay {
            states,
            actions: actions.to_vec(),
            step_height,
            summary,
            summary_height,
            max_step: actions.len() as isize,
            term_size,
            state_height,
        }
    }

    pub fn run(&self) -> Result<(), io::Error> {
        let stdin = io::stdin();

        let backend = TermionBackend::new()?;
        let mut terminal = Terminal::new(backend)?;

        terminal.resize(self.term_size)?;

        terminal.hide_cursor()?;
        terminal.clear()?;

        let mut step = 0;

        self.draw(step, &mut terminal)?;
        for c in stdin.keys() {
            let evt = c?;

            match evt {
                event::Key::Esc => break,
                event::Key::Right => step = self.trim_step(step + 1),
                event::Key::Left => step = self.trim_step(step - 1),
                _ => (),
            }

            self.draw(step, &mut terminal)?;
        }

        // Force a full re-draw so that it the cursor is at the end.
        terminal.resize(self.term_size)?;
        self.draw(step, &mut terminal)?;

        terminal.show_cursor()?;

        Ok(())
    }

    fn trim_step(&self, step: isize) -> isize {

        if step < 0 {
            0
        } else if step > self.max_step {
            self.max_step
        } else {
            step
        }
    }

    pub fn draw(&self, step: isize, mut t: &mut Terminal<TermionBackend>) -> Result<(), io::Error> {

        Group::default()
            .direction(Direction::Vertical)
            .sizes(
                &[
                    Size::Fixed(self.state_height),
                    Size::Fixed(self.step_height),
                    Size::Fixed(self.summary_height),
                ],
            )
            .render(&mut t, &self.term_size, |mut t, chunks| {

                let render_state_chunk = chunks[0];
                let step_data_chunk = chunks[1];
                let run_data_chunk = chunks[2];

                Paragraph::default()
                    .text(&self.states[step as usize])
                    .render(&mut t, &render_state_chunk);


                let step_data = build_step_str(step as usize, &self.actions);

                Paragraph::default().text(&step_data).render(
                    &mut t,
                    &step_data_chunk,
                );


                Paragraph::default().wrap(true).text(&self.summary).render(
                    &mut t,
                    &run_data_chunk,
                );
            });

        t.draw()?;

        Ok(())
    }
}

fn build_step_str(step: usize, actions: &[Actions]) -> String {
    let mut result = String::new();
    result += &format!("Step {:^3}\n", step);

    if step < actions.len() {
        result += &format!("Next action: {}", actions[step]);
    } else {
        result += "Finished"
    }

    result
}
