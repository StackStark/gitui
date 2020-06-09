use super::{
    command_pump, event_pump, visibility_blocking, CommandBlocking,
    CommandInfo, CommitDetailsComponent, Component, DiffComponent,
    DrawableComponent,
};
use crate::{
    accessors, keys, queue::Queue, strings, ui::style::Theme,
};
use anyhow::Result;
use asyncgit::{sync, AsyncNotification, CWD};
use crossbeam_channel::Sender;
use crossterm::event::Event;
use strings::commands;
use sync::{CommitId, Tags};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
    Frame,
};

pub struct InspectCommitComponent {
    commit_id: Option<CommitId>,
    diff: DiffComponent,
    details: CommitDetailsComponent,
    visible: bool,
}

impl DrawableComponent for InspectCommitComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> Result<()> {
        if self.is_visible() {
            let percentages = if self.diff.focused() {
                (30, 70)
            } else {
                (50, 50)
            };

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(percentages.0),
                        Constraint::Percentage(percentages.1),
                    ]
                    .as_ref(),
                )
                .split(rect);

            f.render_widget(Clear, rect);

            self.details.draw(f, chunks[0])?;
            self.diff.draw(f, chunks[1])?;
        }

        Ok(())
    }
}

impl Component for InspectCommitComponent {
    fn commands(
        &self,
        out: &mut Vec<CommandInfo>,
        force_all: bool,
    ) -> CommandBlocking {
        if self.is_visible() || force_all {
            command_pump(
                out,
                force_all,
                self.components().as_slice(),
            );
        }

        out.push(
            CommandInfo::new(
                commands::CLOSE_POPUP,
                true,
                self.is_visible(),
            )
            .order(1),
        );

        visibility_blocking(self)
    }

    fn event(&mut self, ev: Event) -> Result<bool> {
        if self.is_visible() {
            if event_pump(ev, self.components_mut().as_mut_slice())? {
                return Ok(true);
            }

            if let Event::Key(e) = ev {
                match e {
                    keys::EXIT_POPUP => {
                        self.hide();
                    }
                    keys::FOCUS_RIGHT if self.can_focus_diff() => {
                        self.details.focus(false);
                        self.diff.focus(true);
                    }
                    keys::FOCUS_LEFT if self.diff.focused() => {
                        self.details.focus(true);
                        self.diff.focus(false);
                    }
                    _ => (),
                }

                // stop key event propagation
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
    fn hide(&mut self) {
        self.visible = false;
    }
    fn show(&mut self) -> Result<()> {
        self.visible = true;
        self.details.show()?;
        self.details.focus(true);
        self.diff.focus(false);
        self.update()?;
        Ok(())
    }
}

impl InspectCommitComponent {
    accessors!(self, [diff, details]);

    ///
    pub fn new(
        queue: &Queue,
        sender: &Sender<AsyncNotification>,
        theme: &Theme,
    ) -> Self {
        Self {
            details: CommitDetailsComponent::new(
                queue, sender, theme,
            ),
            diff: DiffComponent::new(None, theme),
            commit_id: None,
            visible: false,
        }
    }

    ///
    pub fn open(&mut self, id: CommitId) -> Result<()> {
        self.commit_id = Some(id);
        self.show()?;

        Ok(())
    }

    ///
    pub fn any_work_pending(&self) -> bool {
        self.details.any_work_pending()
    }

    ///
    pub fn update_git(
        &mut self,
        ev: AsyncNotification,
    ) -> Result<()> {
        if self.is_visible() {
            if let AsyncNotification::CommitFiles = ev {
                self.update()?
            } else if let AsyncNotification::Diff = ev {
                self.update()?
            }
        }

        Ok(())
    }

    /// called when any tree component changed selection
    pub fn update_diff(&mut self) -> Result<()> {
        if self.is_visible() {
            if let Some(id) = self.commit_id {
                if let Some(f) = self.details.files().selection_file()
                {
                    self.diff.update(
                        f.path.clone(),
                        false,
                        sync::get_diff_commit(CWD, id, f.path)?,
                    )?;

                    return Ok(());
                }
            }

            self.diff.clear()?;
        }

        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        self.details.set_commit(self.commit_id, &Tags::new())?;
        self.update_diff()?;

        Ok(())
    }

    fn can_focus_diff(&self) -> bool {
        self.details.files().selection_file().is_some()
    }
}