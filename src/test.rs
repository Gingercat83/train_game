use crate::{
    signal::{
        DECELERATION, Signal, SignalState, Signals, SpeedControl, SpeedSign, SpeedSignType,
        calc_start_speed, calc_stopping_distance,
    },
    station::generate_station_name,
    *,
};

#[test]
fn generate_station_names() {
    for _ in 0..50 {
        let name = generate_station_name();
        println!("{name}");
    }
}

#[test]
fn render_signs() {
    clearscreen::clear().unwrap();

    SpeedSign::new(SpeedSignType::Limit(40), 40.0)
        .render(0.0)
        .unwrap();

    SpeedSign::new(SpeedSignType::Limit(160), 80.0)
        .render(0.0)
        .unwrap();

    SpeedSign::new(SpeedSignType::Reduction(120), 120.0)
        .render(0.0)
        .unwrap();

    SpeedSign::new(SpeedSignType::Reduction(60), 160.0)
        .render(0.0)
        .unwrap();
}

#[test]
fn render_signals() {
    use SignalState as S;

    clearscreen::clear().unwrap();
    let mut sig = Signal::new(240.0);

    sig.render(210.0).unwrap();

    sig.state = S::Yellow(SpeedControl {
        limit: None,
        reduction: Some(0),
        is_permanent: false,
    });
    sig.render(180.0).unwrap();

    sig.state = S::Yellow(SpeedControl {
        limit: Some(40),
        reduction: Some(40),
        is_permanent: false,
    });
    sig.render(150.0).unwrap();

    sig.state = S::Yellow(SpeedControl {
        limit: Some(100),
        reduction: Some(60),
        is_permanent: false,
    });
    sig.render(120.0).unwrap();

    sig.state = S::Green(SpeedControl {
        limit: None,
        reduction: None,
        is_permanent: false,
    });
    sig.render(90.0).unwrap();

    sig.state = S::Green(SpeedControl {
        limit: Some(60),
        reduction: Some(100),
        is_permanent: false,
    });
    sig.render(60.0).unwrap();

    sig.state = S::Green(SpeedControl {
        limit: None,
        reduction: Some(120),
        is_permanent: false,
    });
    sig.render(30.0).unwrap();
}

#[test]
fn calc_stopping_distances() {
    let start_speed = calc_start_speed(0.0, 1000.0, DECELERATION);
    println!("stop from 1km: {start_speed:.2}");

    let start_speed = calc_start_speed(0.0, 500.0, DECELERATION);
    println!("stop from 500m: {start_speed:.2}");

    let start_speed = calc_start_speed(110.0, 500.0, DECELERATION);
    println!("to 110kph from 500m: {start_speed:.2}");

    let start_speed = calc_start_speed(0.0, 200.0, DECELERATION);
    println!("stop from 200m: {start_speed:.2}");

    let start_speed = calc_start_speed(40.0, 200.0, DECELERATION);
    println!("to 40kph from 200m: {start_speed:.2}");

    let start_speed = calc_start_speed(60.0, 200.0, DECELERATION);
    println!("to 60kph from 200m: {start_speed:.2}");

    let start_speed = calc_start_speed(100.0, 200.0, DECELERATION);
    println!("to 100kph from 200m: {start_speed:.2}");

    println!("---");

    let dist = calc_stopping_distance(0.0, 160.0, DECELERATION);
    println!("160kph to stop: {dist:.0}m");

    let dist = calc_stopping_distance(0.0, 110.0, DECELERATION);
    println!("110kph to stop: {dist:.0}m");

    let dist = calc_stopping_distance(110.0, 160.0, DECELERATION);
    println!("160kph to 110kph: {dist:.0}m");

    let dist = calc_stopping_distance(0.0, 40.0, DECELERATION);
    println!("40kph to stop: {dist:.0}m");

    let dist = calc_stopping_distance(0.0, 60.0, DECELERATION);
    println!("60kph to stop: {dist:.0}m");
}

#[test]
fn test_signals() {
    clearscreen::clear().unwrap();

    let mut signals = Signals::new();

    for i in 0..5 {
        let pos = 400.0 - (i * 80) as f64;

        signals.add_signal(Signal::new(pos));
    }

    signals.set_signal_speed_limit(3, Some(60)).unwrap();
    signals.set_signal_speed_limit(4, Some(150)).unwrap();

    // this will also call update_signals()
    signals.clear_all_signals();

    signals.render(0.0).unwrap();
}

#[test]
fn overlap() {
    // a is before b but not overlapping
    assert_eq!(is_overlapping((0.0, 1.0), (2.0, 3.0)), false);
    // a overlaps b
    assert_eq!(is_overlapping((0.0, 2.0), (1.0, 3.0)), true);
    // a contains b
    assert_eq!(is_overlapping((0.0, 3.0), (1.0, 2.0)), true);
    // b is before a but not overlapping
    assert_eq!(is_overlapping((2.0, 3.0), (0.0, 1.0)), false);
    // b overlaps a
    assert_eq!(is_overlapping((1.0, 3.0), (0.0, 2.0)), true);
    // b contains a
    assert_eq!(is_overlapping((1.0, 2.0), (0.0, 3.0)), true);
}

// #[test]
// fn alignment_from_str() {
//     use render::*;
//     assert_eq!(
//         Alignment::from_str("tl"),
//         Alignment {
//             vertical: VerticalAlignment::Top,
//             horizontal: HorizontalAlignment::Left
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("tr"),
//         Alignment {
//             vertical: VerticalAlignment::Top,
//             horizontal: HorizontalAlignment::Right
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("bl"),
//         Alignment {
//             vertical: VerticalAlignment::Bottom,
//             horizontal: HorizontalAlignment::Left
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("br"),
//         Alignment {
//             vertical: VerticalAlignment::Bottom,
//             horizontal: HorizontalAlignment::Right
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("tc"),
//         Alignment {
//             vertical: VerticalAlignment::Top,
//             horizontal: HorizontalAlignment::Center
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("b"),
//         Alignment {
//             vertical: VerticalAlignment::Bottom,
//             horizontal: HorizontalAlignment::Center
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("rc"),
//         Alignment {
//             vertical: VerticalAlignment::Center,
//             horizontal: HorizontalAlignment::Right
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("cl"),
//         Alignment {
//             vertical: VerticalAlignment::Center,
//             horizontal: HorizontalAlignment::Left
//         }
//     );
//     assert_eq!(
//         Alignment::from_str("c"),
//         Alignment {
//             vertical: VerticalAlignment::Center,
//             horizontal: HorizontalAlignment::Center
//         }
//     );
// }
