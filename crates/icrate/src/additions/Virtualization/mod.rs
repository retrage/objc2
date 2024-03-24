#![allow(unused_imports)]
use crate::generated::Virtualization::*;

objc2::extern_methods!(
    unsafe impl VZVirtualMachine {
        #[cfg(feature = "Virtualization_VZVirtualMachineConfiguration")]
        #[method_id(@__retain_semantics Init initWithConfiguration:queue:)]
        pub unsafe fn initWithConfiguration_queue(
            this: objc2::rc::Allocated<Self>,
            configuration: &VZVirtualMachineConfiguration,
            queue: &crate::queue::dispatch_queue_t,
        ) -> objc2::rc::Id<Self>;
    }
);
