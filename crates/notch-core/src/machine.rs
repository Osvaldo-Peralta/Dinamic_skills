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
        todo!()
    }

    pub fn set_player_present(&mut self, present: bool, now: Instant) {
        todo!()
    }

    pub fn on_hover(&mut self, ev: HoverEvent, now: Instant) {
        todo!()
    }

    pub fn tick(&mut self, now: Instant) -> NotchState {
        todo!()
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
}
