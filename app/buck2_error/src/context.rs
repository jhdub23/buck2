/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::sync::Arc;

use crate::context_value::ContextValue;
use crate::error::ErrorKind;

impl crate::Error {
    pub fn context<C: Into<ContextValue>>(self, context: C) -> Self {
        Self(Arc::new(ErrorKind::WithContext(context.into(), self)))
    }

    #[cold]
    #[track_caller]
    fn new_anyhow_with_context<E, C: Into<ContextValue>>(e: E, c: C) -> anyhow::Error
    where
        crate::Error: From<E>,
    {
        crate::Error::from(e).context(c).into()
    }
}

/// Provides the `context` method for `Result`.
///
/// This trait is analogous to the `anyhow::Context` trait. It is mostly a drop-in replacement, and
/// in the near future, uses of `anyhow::Context` in `buck2/app` will be broadly replaced with use
/// of this trait. Subsequently, additional APIs will be provided for annotating errors with
/// structured context data.
pub trait Context<T>: Sealed {
    #[track_caller]
    fn context<C: Into<ContextValue>>(self, context: C) -> anyhow::Result<T>;

    #[track_caller]
    fn with_context<C, F>(self, f: F) -> anyhow::Result<T>
    where
        C: Into<ContextValue>,
        F: FnOnce() -> C;

    #[track_caller]
    fn user(self) -> anyhow::Result<T> {
        self.context(crate::Category::User)
    }

    #[track_caller]
    fn infra(self) -> anyhow::Result<T> {
        self.context(crate::Category::Infra)
    }
}
pub trait Sealed: Sized {}

impl<T, E> Sealed for std::result::Result<T, E> where crate::Error: From<E> {}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    crate::Error: From<E>,
{
    fn context<C>(self, c: C) -> anyhow::Result<T>
    where
        C: Into<ContextValue>,
    {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(crate::Error::new_anyhow_with_context(e, c)),
        }
    }

    fn with_context<C, F>(self, f: F) -> anyhow::Result<T>
    where
        C: Into<ContextValue>,
        F: FnOnce() -> C,
    {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(crate::Error::new_anyhow_with_context(e, f())),
        }
    }
}

// FIXME(JakobDegen): This impl should not exist. We should make people write error types for these
// conditions. Let's have it for now to ease migration though.

#[derive(Debug, buck2_error_derive::Error)]
#[error("NoneError")]
struct NoneError;

impl<T> Sealed for Option<T> {}

impl<T> Context<T> for Option<T> {
    fn context<C>(self, c: C) -> anyhow::Result<T>
    where
        C: Into<ContextValue>,
    {
        match self {
            Some(x) => Ok(x),
            None => Err(crate::Error::new_anyhow_with_context(NoneError, c)),
        }
    }

    fn with_context<C, F>(self, f: F) -> anyhow::Result<T>
    where
        C: Into<ContextValue>,
        F: FnOnce() -> C,
    {
        match self {
            Some(x) => Ok(x),
            None => Err(crate::Error::new_anyhow_with_context(NoneError, f())),
        }
    }
}
