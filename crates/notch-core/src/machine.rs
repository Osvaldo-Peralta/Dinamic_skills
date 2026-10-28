//! Máquina de estados del notch: Hidden / Resting / Expanded.
//! El tiempo entra siempre como parámetro (sin reloj interno).

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotchState {
    Hidden,
    Resting,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverEvent {
    Enter,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pending {
    Expand(Instant),
    Collapse(Instant),
}

pub struct NotchMachine {
    enter_delay: Duration,
    leave_delay: Duration,
    player_present: bool,
    hovered: bool,
    state: NotchState,
    pending: Option<Pending>,
}

impl NotchMachine {
    pub fn new(enter_delay: Duration, leave_delay: Duration) -> Self {
        Self {
            enter_delay,
            leave_delay,
            player_present: false,
            hovered: false,
            state: NotchState::Hidden,
            pending: None,
        }
    }

    pub fn set_player_present(&mut self, present: bool, _now: Instant) {
        if self.player_present == present {
            return;
        }
        self.player_present = present;
        self.pending = None;
        if present {
            self.state = NotchState::Resting;
        } else {
            self.state = NotchState::Hidden;
            self.hovered = false;
        }
    }

    pub fn on_hover(&mut self, ev: HoverEvent, now: Instant) {
        // Sin reproductor no hay notch: los eventos se ignoran por completo,
        // sin tocar `hovered`.
        if !self.player_present {
            return;
        }
        match ev {
            HoverEvent::Enter => {
                self.hovered = true;
                match self.state {
                    NotchState::Resting => {
                        self.pending = Some(Pending::Expand(now + self.enter_delay));
                    }
                    NotchState::Expanded => {
                        // Reentrar durante el retardo de salida cancela el colapso.
                        if matches!(self.pending, Some(Pending::Collapse(_))) {
                            self.pending = None;
                        }
                    }
                    NotchState::Hidden => {}
                }
            }
            HoverEvent::Leave => {
                self.hovered = false;
                // Un paso rápido cancela la expansión pendiente.
                if matches!(self.pending, Some(Pending::Expand(_))) {
                    self.pending = None;
                    return;
                }
                if self.state == NotchState::Expanded {
                    self.pending = Some(Pending::Collapse(now + self.leave_delay));
                }
            }
        }
    }

    pub fn tick(&mut self, now: Instant) -> NotchState {
        if !self.player_present {
            self.state = NotchState::Hidden;
            self.pending = None;
            return self.state;
        }
        match self.pending {
            Some(Pending::Expand(deadline)) if now >= deadline && self.hovered => {
                self.state = NotchState::Expanded;
                self.pending = None;
            }
            Some(Pending::Collapse(deadline)) if now >= deadline && !self.hovered => {
                self.state = NotchState::Resting;
                self.pending = None;
            }
            _ => {}
        }
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn machine() -> (NotchMachine, Instant) {
        (
            NotchMachine::new(Duration::from_millis(150), Duration::from_millis(400)),
            Instant::now(),
        )
    }

    #[test]
    fn hover_enter_expands_after_150ms() {
        let (mut m, t0) = machine();
        m.set_player_present(true, t0);
        assert_eq!(m.tick(t0), NotchState::Resting);
        m.on_hover(HoverEvent::Enter, t0);
        assert_eq!(m.tick(t0 + Duration::from_millis(100)), NotchState::Resting);
        assert_eq!(m.tick(t0 + Duration::from_millis(150)), NotchState::Expanded);
    }

    #[test]
    fn hover_leave_collapses_after_400ms() {
        let (mut m, t0) = machine();
        m.set_player_present(true, t0);
        m.on_hover(HoverEvent::Enter, t0);
        let t1 = t0 + Duration::from_millis(150);
        assert_eq!(m.tick(t1), NotchState::Expanded);
        m.on_hover(HoverEvent::Leave, t1);
        assert_eq!(m.tick(t1 + Duration::from_millis(399)), NotchState::Expanded);
        assert_eq!(m.tick(t1 + Duration::from_millis(400)), NotchState::Resting);
    }

    #[test]
    fn quick_pass_does_not_expand() {
        let (mut m, t0) = machine();
        m.set_player_present(true, t0);
        assert_eq!(m.tick(t0), NotchState::Resting);
        m.on_hover(HoverEvent::Enter, t0);
        m.on_hover(HoverEvent::Leave, t0 + Duration::from_millis(100));
        for ms in [100u64, 150, 200, 600, 1000] {
            assert_eq!(
                m.tick(t0 + Duration::from_millis(ms)),
                NotchState::Resting,
                "un paso rápido nunca debe llegar a Expanded"
            );
        }
    }

    #[test]
    fn reenter_during_leave_delay_cancels_collapse() {
        let (mut m, t0) = machine();
        m.set_player_present(true, t0);
        m.on_hover(HoverEvent::Enter, t0);
        let t1 = t0 + Duration::from_millis(150);
        assert_eq!(m.tick(t1), NotchState::Expanded);
        m.on_hover(HoverEvent::Leave, t1);
        m.on_hover(HoverEvent::Enter, t1 + Duration::from_millis(100));
        assert_eq!(m.tick(t1 + Duration::from_millis(500)), NotchState::Expanded);
    }

    #[test]
    fn no_player_is_hidden() {
        let (mut m, t0) = machine();
        assert_eq!(m.tick(t0), NotchState::Hidden);
        m.on_hover(HoverEvent::Enter, t0);
        assert_eq!(m.tick(t0 + Duration::from_millis(1000)), NotchState::Hidden);
    }

    #[test]
    fn player_appears_goes_to_resting() {
        let (mut m, t0) = machine();
        assert_eq!(m.tick(t0), NotchState::Hidden);
        m.set_player_present(true, t0);
        assert_eq!(m.tick(t0), NotchState::Resting);
    }

    #[test]
    fn hover_ignored_while_no_player() {
        let (mut m, t0) = machine();
        m.on_hover(HoverEvent::Enter, t0);
        m.set_player_present(true, t0);
        assert_eq!(m.tick(t0 + Duration::from_secs(1)), NotchState::Resting);
    }

    #[test]
    fn transition_pending_is_true_only_while_delay_is_running() {
        let (mut m, t0) = machine();
        m.set_player_present(true, t0);
        assert!(!m.transition_pending(), "en reposo no hay transición pendiente");
        m.on_hover(HoverEvent::Enter, t0);
        assert!(m.transition_pending(), "durante el retardo de entrada");
        m.tick(t0 + Duration::from_millis(150));
        assert!(!m.transition_pending(), "tras expandirse no queda pendiente");
        m.on_hover(HoverEvent::Leave, t0 + Duration::from_millis(150));
        assert!(m.transition_pending(), "durante el retardo de salida");
        m.tick(t0 + Duration::from_millis(150 + 400));
        assert!(!m.transition_pending(), "tras colapsar no queda pendiente");
    }

    #[test]
    fn player_disappearing_while_expanded_hides_then_resting() {
        let (mut m, t0) = machine();
        m.set_player_present(true, t0);
        m.on_hover(HoverEvent::Enter, t0);
        let t1 = t0 + Duration::from_millis(150);
        assert_eq!(m.tick(t1), NotchState::Expanded);
        m.set_player_present(false, t1);
        assert_eq!(m.tick(t1), NotchState::Hidden);
        m.set_player_present(true, t1 + Duration::from_millis(100));
        assert_eq!(
            m.tick(t1 + Duration::from_millis(100)),
            NotchState::Resting
        );
    }
}
