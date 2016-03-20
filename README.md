Objective-C Runtime bindings and wrapper for Rust.

* Documentation: http://ssheldon.github.io/rust-objc/objc/
* Crate: https://crates.io/crates/objc

## Messaging objects

Objective-C objects can be messaged using the `msg_send!` macro:

``` rust
let cls = Class::get("NSObject").unwrap();
let obj: *mut Object = msg_send![cls, new];
let hash: usize = msg_send![obj, hash];
let is_kind: BOOL = msg_send![obj, isKindOfClass:cls];
// Even void methods must have their return type annotated
let _: () = msg_send![obj, release];
```

## Declaring classes

Classes can be declared using the `ClassDecl` struct. Instance variables and
methods can then be added before the class is ultimately registered.

The following example demonstrates declaring a class named `MyNumber` that has
one ivar, a `u32` named `_number` and a `number` method that returns it:

``` rust
let superclass = Class::get("NSObject").unwrap();
let mut decl = ClassDecl::new("MyNumber", superclass).unwrap();

// Add an instance variable
decl.add_ivar::<u32>("_number");

// Add an ObjC method for getting the number
extern fn my_number_get(this: &Object, _cmd: Sel) -> u32 {
    unsafe { *this.get_ivar("_number") }
}
unsafe {
    decl.add_method(sel!(number),
        my_number_get as extern fn(&Object, Sel) -> u32);
}

decl.register();
```

## Exceptions

By default, if the `msg_send!` macro causes an exception to be thrown, this
will unwind into Rust resulting in unsafe, undefined behavior.
However, this crate has an `"exception"` feature which, when enabled, wraps
each `msg_send!` in a `@try`/`@catch` and panics if an exception is caught,
preventing Objective-C from unwinding into Rust.

## Support for other Operating Systems

The bindings can be used on Linux or *BSD utilizing the
[GNUstep Objective-C runtime](https://www.github.com/gnustep/libobjc2).
