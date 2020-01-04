use std::io;

use crossterm::event;
use crossterm::event::{Event, KeyCode};

use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Paragraph, Text, Widget};
use tui::Terminal;

use taxi::actions::Actions;
use taxi::runner::Attempt;
use taxi::world::World;

pub struct Replay {
    states: Vec<String>,
    actions: Vec<Actions>,
    solved: bool,
    step_height: u16,
    summary_height: u16,
    summary: String,
    max_step: isize,
    term_size: Rect,
    state_height: u16,
}

impl Replay {
    pub fn new(world: &World, attempt: Attempt) -> Replay {
        let mut states = Vec::with_capacity(attempt.actions.len() + 1);
        let mut state = attempt.initial_state;
        states.push(state.display(world));

        for a in &attempt.actions {
            let (_, next_state) = state.apply_action(world, *a);
            states.push(next_state.display(world));
            state = next_state;
        }

        let state_height = (2 * world.height + 1) as u16;

        let num_actions = attempt.actions.len();

        let summary = build_summary_string(attempt.success, num_actions);

        let summary_height = summary.lines().count() as u16;

        let step_height = 3;

        let term_width = 80;
        let term_height = state_height + summary_height + step_height;

        let term_size = Rect::new(0, 0, term_width, term_height);

        Replay {
            states,
            actions: attempt.actions,
            solved: attempt.success,
            step_height,
            summary,
            summary_height,
            max_step: num_actions as isize,
            term_size,
            state_height,
        }
    }

    pub fn run(&self) -> Result<(), io::Error> {
        let stdout = io::stdout();

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.resize(self.term_size)?;

        terminal.hide_cursor()?;
        terminal.clear()?;

        let mut step = 0;

        self.draw(step, &mut terminal)?;

        loop {
            if let Ok(event) = event::read() {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Right => step = self.trim_step(step + 1),
                        KeyCode::Left => step = self.trim_step(step - 1),
                        _ => (),
                    }
                }
            };

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

    pub fn draw(
        &self,
        step: isize,
        t: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), io::Error> {
        t.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(self.state_height),
                        Constraint::Length(self.step_height),
                        Constraint::Length(self.summary_height),
                    ]
                    .as_ref(),
                )
                .split(self.term_size);

            let step_data = build_step_string(step as usize, self.solved, &self.actions);

            Paragraph::new([Text::raw(&self.states[step as usize])].iter())
                .wrap(true)
                .render(&mut f, chunks[0]);

            Paragraph::new([Text::raw(&step_data)].iter())
                .wrap(true)
                .render(&mut f, chunks[1]);

            Paragraph::new([Text::raw(&self.summary)].iter())
                .wrap(true)
                .render(&mut f, chunks[2]);
        })?;

        Ok(())
    }
}

fn build_summary_string(solved: bool, num_steps: usize) -> String {
    let mut result = String::new();

    if solved {
        result += &format!("Solved in {} steps.", num_steps);
    } else {
        result += &format!("Failed after {} steps.", num_steps);
    };

    result += "\n\
               \n\
               Left/Right arrow to advance.\n\
               Escape to exit.\n\
               ";

    result
}

fn build_step_string(step: usize, solved: bool, actions: &[Actions]) -> String {
    let mut result = String::new();
    result += &format!("Step {:^3}", step);

    if step < actions.len() {
        result += &format!("\nNext action: {}", actions[step]);
    } else if solved {
        result += "\nSucceeded";
    } else {
        result += "\nFailed";
    }

    result
}
