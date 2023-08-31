// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Null scheduler that does not run applications.

// use crate::collections::list::{List, ListLink, ListNode};
// use crate::kernel::StoppedExecutingReason;
// use crate::platform::chip::Chip;
// use crate::process::Process;
use kernel::platform::chip::Chip;
use kernel::scheduler;
use kernel::scheduler::SchedulingDecision;

pub struct NullScheduler {}

impl<'a> NullScheduler {
    pub const fn new() -> NullScheduler {
        NullScheduler {}
    }
}

impl<'a, C: Chip> scheduler::Scheduler<C> for NullScheduler {
    fn next(&self) -> SchedulingDecision {
        scheduler::SchedulingDecision::TrySleep
    }

    fn result(&self, _result: kernel::process::StoppedExecutingReason, _: Option<u32>) {}
}
