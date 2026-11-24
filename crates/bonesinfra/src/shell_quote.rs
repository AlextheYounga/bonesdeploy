use std::borrow::Cow;

use shlex::QuoteError;

pub(crate) fn quote(value: &str) -> Result<String, QuoteError> {
    shlex::try_quote(value).map(Cow::into_owned)
}
