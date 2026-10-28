//! Geometría del notch: posición de la píldora y región interactiva.

use crate::machine::NotchState;
use crate::{Rect, Size};

/// Rect de la píldora: centrada en horizontal, pegada al borde superior de
/// `screen`, ajustada a la cuadrícula de píxeles físicos y nunca fuera de
/// `screen`.
pub fn compute_pill_rect(screen: Rect, pill: Size, device_pixel_ratio: f64) -> Rect {
    let dpr = if device_pixel_ratio > 0.0 {
        device_pixel_ratio
    } else {
        1.0
    };
    let snap = |v: f64| (v * dpr).round() / dpr;

    // Nunca más ancha que la pantalla.
    let w = snap(pill.w.min(screen.w)).min(screen.w);
    // Centrada en horizontal sobre el borde superior de `screen`.
    let x_raw = screen.x + (screen.w - w) / 2.0;
    let x = snap(x_raw).clamp(screen.x, screen.x + screen.w - w);
    let y = snap(screen.y).max(screen.y);
    let h = snap(pill.h).min(screen.y + screen.h - y);

    Rect { x, y, w, h }
}

/// Región sensible al mouse según el estado: ninguna si Hidden.
pub fn interactive_region(state: NotchState, resting: Rect, expanded: Rect) -> Option<Rect> {
    match state {
        NotchState::Hidden => None,
        NotchState::Resting => Some(resting),
        NotchState::Expanded => Some(expanded),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pill_is_centered_at_top() {
        let screen = Rect {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        };
        let pill = Size { w: 320.0, h: 44.0 };
        let r = compute_pill_rect(screen, pill, 1.0);
        assert_eq!(r.x, 800.0);
        assert_eq!(r.y, 0.0);
        assert_eq!(r.w, 320.0);
        assert_eq!(r.h, 44.0);
    }

    #[test]
    fn pill_snaps_to_physical_pixels() {
        let screen = Rect {
            x: 0.0,
            y: 0.0,
            w: 1280.0,
            h: 720.0,
        };
        let pill = Size { w: 321.0, h: 44.0 };
        let r = compute_pill_rect(screen, pill, 1.5);
        assert!((r.x * 1.5).fract().abs() < 1e-9);
    }

    #[test]
    fn pill_stays_on_screen_when_screen_shrinks() {
        let screen = Rect {
            x: 0.0,
            y: 0.0,
            w: 300.0,
            h: 800.0,
        };
        let pill = Size { w: 320.0, h: 44.0 };
        let r = compute_pill_rect(screen, pill, 2.0);
        assert!(r.w <= 300.0);
        assert!(r.x >= screen.x);
    }

    #[test]
    fn pill_stays_inside_screen_on_both_edges() {
        let screen = Rect {
            x: 100.0,
            y: 0.0,
            w: 200.0,
            h: 600.0,
        };
        let pill = Size { w: 320.0, h: 44.0 };
        let r = compute_pill_rect(screen, pill, 2.0);
        assert!(r.x >= screen.x);
        assert!(r.x + r.w <= screen.x + screen.w);
    }

    #[test]
    fn interactive_region_matches_state() {
        let resting = Rect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
        };
        let expanded = Rect {
            x: 5.0,
            y: 6.0,
            w: 7.0,
            h: 8.0,
        };
        assert_eq!(interactive_region(NotchState::Hidden, resting, expanded), None);
        assert_eq!(
            interactive_region(NotchState::Resting, resting, expanded),
            Some(resting)
        );
        assert_eq!(
            interactive_region(NotchState::Expanded, resting, expanded),
            Some(expanded)
        );
    }
}
