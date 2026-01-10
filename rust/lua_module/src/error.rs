pub struct AnyError;

impl<T> From<T> for AnyError
where
    T: std::error::Error,
{
    fn from(_value: T) -> Self {
        Self {}
    }
}
