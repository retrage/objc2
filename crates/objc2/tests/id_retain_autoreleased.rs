use core::mem::ManuallyDrop;

use objc2::msg_send;
use objc2::rc::{autoreleasepool, Id, Shared};
use objc2::runtime::NSObject;

#[cfg_attr(feature = "apple", link(name = "Foundation", kind = "framework"))]
#[cfg_attr(feature = "gnustep-1-7", link(name = "gnustep-base", kind = "dylib"))]
extern "C" {}

#[cfg(feature = "gnustep-1-7")]
#[test]
fn ensure_linkage() {
    unsafe { objc2::__gnustep_hack::get_class_to_force_linkage() };
}

fn retain_count(obj: &NSObject) -> usize {
    unsafe { msg_send![obj, retainCount] }
}

fn create_obj() -> Id<NSObject, Shared> {
    let obj = ManuallyDrop::new(NSObject::new());
    unsafe {
        let obj: *mut NSObject = msg_send![&*obj, autorelease];
        // All code between the `msg_send!` and the `retain_autoreleased` must
        // be able to be optimized away for this to work.
        Id::retain_autoreleased(obj).unwrap()
    }
}

#[test]
fn test_retain_autoreleased() {
    autoreleasepool(|_| {
        // Run once to allow DYLD to resolve the symbol stubs.
        // Required for making `retain_autoreleased` work on x86_64.
        let _data = create_obj();

        // When compiled in release mode / with optimizations enabled,
        // subsequent usage of `retain_autoreleased` will succeed in retaining
        // the autoreleased value!
        let expected = if cfg!(feature = "gnustep-1-7") {
            1
        } else if cfg!(any(debug_assertions, feature = "exception")) {
            2
        } else {
            1
        };

        let data = create_obj();
        assert_eq!(retain_count(&data), expected);

        let data = create_obj();
        assert_eq!(retain_count(&data), expected);

        // Here we manually clean up the autorelease, so it will always be 1.
        let data = autoreleasepool(|_| create_obj());
        assert_eq!(retain_count(&data), 1);
    });
}
