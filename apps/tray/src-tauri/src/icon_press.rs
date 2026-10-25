//! On macOS, VoiceOver and the keyboard (Full Keyboard Access, then Control-F8) open QuotaDial
//! from its menu bar icon (#52, #64). They press the icon's button rather than click it, and the tray
//! library answers only clicks, so the button gets an action of its own that does what a click
//! does. A click never reaches the button, which the library covers with a view of its own, and the
//! library shows its menu by setting it on the icon and pressing the button, which a menu takes
//! instead of the action; so the action runs only for a press.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSMenu, NSStatusItem};

/// What a press does.
struct Ivars {
    press: Box<dyn Fn()>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and `IconPress` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "QuotaDialIconPress"]
    #[ivars = Ivars]
    struct IconPress;

    impl IconPress {
        #[unsafe(method(press:))]
        fn press(&self, _sender: Option<&AnyObject>) {
            (self.ivars().press)();
        }
    }
);

thread_local! {
    /// The button's target. AppKit does not keep a control's target alive, so it is kept here, on
    /// the main thread, for as long as the icon.
    static TARGET: RefCell<Option<Retained<IconPress>>> = const { RefCell::new(None) };
}

/// Makes a press of `item`'s button, such as VoiceOver's, call `press`. AppKit is used only on the
/// main thread, so elsewhere this does nothing.
pub fn answer_presses(item: &NSStatusItem, press: impl Fn() + 'static) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(button) = item.button(mtm) else {
        return;
    };
    let target = IconPress::alloc(mtm).set_ivars(Ivars {
        press: Box::new(press),
    });
    // SAFETY: NSObject's `init` on a newly allocated object.
    let target: Retained<IconPress> = unsafe { msg_send![super(target), init] };
    // SAFETY: the target is an object that implements the action, `press:`, which takes the sender.
    unsafe {
        button.setTarget(Some(&target));
        button.setAction(Some(sel!(press:)));
    }
    TARGET.with(|kept| *kept.borrow_mut() = Some(target));
}

/// Presses `item`'s button as the tray library does: with `menu` set on the icon, as it shows its
/// menu after a right click, or with none. Only the press test uses it, to show that a press with a
/// menu set never runs the action `answer_presses` gives the button; the release build leaves it
/// out, unused.
pub fn press_as_the_tray_library_does(item: &NSStatusItem, menu: Option<&NSMenu>) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(button) = item.button(mtm) else {
        return;
    };
    item.setMenu(menu);
    // SAFETY: `performClick:` takes any object, or none, as its sender; this passes none, as the
    // tray library does.
    unsafe { button.performClick(None) };
    item.setMenu(None);
}
