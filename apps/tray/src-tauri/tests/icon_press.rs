//! On macOS, pressing the menu bar icon's button, as VoiceOver and Full Keyboard Access do, opens
//! QuotaDial as a click does, and the press the tray library makes to show its menu does not
//! (#64). AppKit works only on the main thread, which a test harness never gives a test, so this
//! test is its own program (`harness = false` in Cargo.toml): it starts the app's event loop and
//! makes the presses from inside it, on a real status item.
//!
//! When AppKit gives no status item, as in a session with no window server, it says so and passes
//! rather than failing on the session.

#[cfg(target_os = "macos")]
fn main() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::process::exit;
    use std::thread;
    use std::time::Duration;

    use dispatch2::{DispatchQueue, DispatchTime};
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

    let Some(mtm) = MainThreadMarker::new() else {
        panic!(
            "this test must run on the main thread: keep `harness = false` for it in Cargo.toml"
        );
    };
    // A press that never returned would hold the test for ever; this ends it instead.
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(30));
        eprintln!("icon_press: timed out, so a press never returned");
        exit(2);
    });
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    let start = DispatchTime::try_from(Duration::from_millis(500)).expect("a short time");
    DispatchQueue::main()
        .after(start, || {
            let mtm = MainThreadMarker::new().expect("the main queue runs on the main thread");
            let passed = catch_unwind(AssertUnwindSafe(|| presses(mtm))).is_ok();
            exit(if passed { 0 } else { 1 });
        })
        .expect("the main queue takes the work");
    app.run();
}

/// The presses, from inside the running app.
#[cfg(target_os = "macos")]
fn presses(mtm: objc2::MainThreadMarker) {
    use std::cell::Cell;
    use std::rc::Rc;

    use objc2_app_kit::{NSAccessibility, NSMenu, NSStatusBar, NSVariableStatusItemLength};
    use quotadial_tray::icon_press::{answer_presses, press_as_the_tray_library_does};

    let bar = NSStatusBar::systemStatusBar();
    let item = bar.statusItemWithLength(NSVariableStatusItemLength);
    let Some(button) = item.button(mtm) else {
        println!(
            "icon_press: skipped, because AppKit gave this session a status item with no button (no window server?)"
        );
        return;
    };
    let presses = Rc::new(Cell::new(0_u32));
    let counted = Rc::clone(&presses);
    answer_presses(&item, move || counted.set(counted.get() + 1));

    // VoiceOver's press, through the accessibility interface: the action runs once, which in the
    // app toggles the popover.
    button.accessibilityPerformPress();
    assert_eq!(
        presses.get(),
        1,
        "an accessibility press of the icon runs its action once"
    );

    // The tray library's own press with no menu set reaches the action too.
    press_as_the_tray_library_does(&item, None);
    assert_eq!(presses.get(), 2, "a press with no menu set runs the action");

    // The same press with a menu set, as the library shows its menu after a right click: the menu
    // takes the press and the action does not run, so a right click never toggles the popover as
    // well. The menu is empty, so AppKit shows nothing and the test leaves no menu open; #64's
    // probe showed the same with a menu that opened.
    let menu = NSMenu::new(mtm);
    press_as_the_tray_library_does(&item, Some(&menu));
    assert_eq!(
        presses.get(),
        2,
        "a press with a menu set does not run the action"
    );

    bar.removeStatusItem(&item);
    println!("icon_press: a press opens QuotaDial, and a press that goes to the menu does not");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
