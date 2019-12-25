#[derive(Debug, FromPrimitive)]
#[repr(i16)]
pub enum Filter {
    Read = -1,
    Write = -2,
    Aio = -3,
    VNode = -4,
    Proc = -5,
    Signal = -6,
    Timer = -7,
    Machport = -8,
    Fs = -9,
    User = -10,
    Vm = -12,
}
