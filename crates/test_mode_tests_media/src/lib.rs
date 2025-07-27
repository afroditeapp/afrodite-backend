#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

mod send_content;

// TODO(test): ProfileContentVersion tests which test API route and
// ProfileLink updating.

pub fn call_this_to_make_sure_that_crate_is_linked() {}
