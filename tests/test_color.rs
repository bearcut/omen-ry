use omen_ry::color::{Color, Hsv};

#[test]
fn test_color_hex_parsing() {
    assert_eq!(Color::from_hex("FF00AA").unwrap(), Color::new(255, 0, 170));
    assert_eq!(Color::from_hex("#FF00AA").unwrap(), Color::new(255, 0, 170));
    assert_eq!(Color::from_hex("000000").unwrap(), Color::new(0, 0, 0));
    assert_eq!(Color::from_hex("#ffffff").unwrap(), Color::new(255, 255, 255));

    // Invalid lengths or characters
    assert!(Color::from_hex("FFF").is_err());
    assert!(Color::from_hex("GGGGGG").is_err());
    assert!(Color::from_hex("#12345").is_err());
}

#[test]
fn test_color_formatting() {
    let c = Color::new(0x0A, 0xBC, 0xDE);
    assert_eq!(c.to_hex(), "0ABCDE");
    assert_eq!(c.to_hex_hash(), "#0ABCDE");
}

#[test]
fn test_color_interpolation() {
    let c1 = Color::new(0, 0, 0);
    let c2 = Color::new(100, 200, 50);

    assert_eq!(c1.lerp(&c2, 0.0), c1);
    assert_eq!(c1.lerp(&c2, 1.0), c2);
    assert_eq!(c1.lerp(&c2, 0.5), Color::new(50, 100, 25));
}

#[test]
fn test_brightness_scaling() {
    let c = Color::new(200, 100, 50);
    assert_eq!(c.scale_brightness(100), c);
    assert_eq!(c.scale_brightness(50), Color::new(100, 50, 25));
    assert_eq!(c.scale_brightness(0), Color::new(0, 0, 0));
}

#[test]
fn test_hsv_to_rgb() {
    // Red: H=0, S=1, V=1
    let red = Hsv::new(0.0, 1.0, 1.0).to_rgb();
    assert_eq!(red, Color::new(255, 0, 0));

    // Green: H=120, S=1, V=1
    let green = Hsv::new(120.0, 1.0, 1.0).to_rgb();
    assert_eq!(green, Color::new(0, 255, 0));

    // Blue: H=240, S=1, V=1
    let blue = Hsv::new(240.0, 1.0, 1.0).to_rgb();
    assert_eq!(blue, Color::new(0, 0, 255));
}
