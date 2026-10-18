use std::time::Duration;
use omen_ry::animations::static_color::StaticColorAnimation;
use omen_ry::animations::wave::WaveAnimation;
use omen_ry::animations::Animation;
use omen_ry::color::Color;

#[test]
fn test_wave_animation_frames() {
    let mut wave = WaveAnimation::new("cyberpunk", 1.0, false);
    assert_eq!(wave.name(), "wave");
    assert!(wave.is_animating());

    let frame1 = wave.tick(Duration::from_millis(16));
    let frame2 = wave.tick(Duration::from_millis(16));

    // Colors should change across frames as time advances
    assert_ne!(frame1, frame2);
}

#[test]
fn test_static_color_quiescence() {
    let mut static_anim = StaticColorAnimation::solid(Color::new(255, 0, 0));
    assert_eq!(static_anim.name(), "static");

    // Before rendering first frame, it needs to render
    assert!(static_anim.is_animating());

    let frame = static_anim.tick(Duration::from_millis(16));
    assert_eq!(frame, [Color::new(255, 0, 0); 4]);

    // After rendering once, is_animating MUST become false to stop the event loop timer!
    assert!(!static_anim.is_animating());

    // Changing color should wake up the animation for one frame
    static_anim.set_zone(0, Color::new(0, 255, 0));
    assert!(static_anim.is_animating());
    let frame_updated = static_anim.tick(Duration::from_millis(16));
    assert_eq!(frame_updated[0], Color::new(0, 255, 0));
    assert!(!static_anim.is_animating());
}
