pub mod objdef;
pub mod statedef;

#[macro_export]
macro_rules! ptr_get {
    ($self:ident,$stack:ident) => {{
        if let Some(stk) = $self.$stack {
            let ptr = stk.as_ptr();
            if !ptr.is_null() {
                Ok(unsafe { &mut *ptr })
            } else {
                Err(ErrCode(MEMORY_UNREACHABLE))
            }
        } else {
            Err(ErrCode(MEMORY_UNREACHABLE))
        }
    }};
    ($sth:expr) => {
        &mut $sth
    };
}

#[macro_export]
macro_rules! ptr_init {
    ($content:expr) => {
        NonNull::from($content)
    };
    ($content:ident) => {
        NonNull::new($content)
    };
    ($self:ident,$ci:ident) => {
        NonNull::from(&$self.$ci)
    };
}




