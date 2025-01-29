use crate::{
    hold, hold_and_release,
    quickjs::{Ctx, Function, Object, Value},
    to_js_error, Args,
};
use anyhow::{anyhow, bail, Error, Result};

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    #[cfg_attr(target_arch = "wasm32", link_name = "sock_accept")]
    fn sock_accept(sockfd: i32, fdflags: i32, socket_offset: i32) -> i32;
    #[cfg_attr(target_arch = "wasm32", link_name = "sock_recv")]
    fn sock_recv(
        sockfd: i32,
        iov_buf: i32,
        iov_buf_len: i32,
        si_flags: i32,
        offset0: i32,
        offset1: i32,
    ) -> i32;
    #[cfg_attr(target_arch = "wasm32", link_name = "sock_send")]
    fn sock_send(sockfd: i32, iov_buf: i32, iov_buf_len: i32, si_flags: i32, offset0: i32) -> i32;
}

pub(crate) fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();
    if globals.get::<_, Object>("net").is_err() {
        globals.set("net", Object::new(this.clone())?)?
    }

    globals.set(
        "__net_socket_accept",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            accept(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    globals.set(
        "__net_socket_read",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            read(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    globals.set(
        "__net_socket_write",
        Function::new(this.clone(), |cx, args| {
            let (cx, args) = hold_and_release!(cx, args);
            write(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
        }),
    )?;

    this.eval::<(), _>(include_str!("net.js"))?;
    Ok::<_, Error>(())
}

// Accept a new connection
fn accept(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, _) = args.release();

    let sock_offset = 0;
    let res = unsafe { sock_accept(4, 0, (&sock_offset as *const i32) as i32) };
    if res != 0 {
        bail!("sock_accept failed with code {}", res);
    }

    Ok(Value::new_number(cx, sock_offset as f64))
}

/// A region of memory for scatter/gather writes.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct IoVec {
    /// Base address of the buffer.
    buf: i32,
    /// Length of the buffer.
    buf_len: u32,
}

/// Read incoming data
fn read(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, args) = args.release();
    let sockfd = args
        .get(0)
        .ok_or_else(|| anyhow!("Expected sockfd"))?
        .as_int()
        .ok_or_else(|| anyhow!("sockfd must be an integer"))? as i32;

    let buffer = args
        .get(1)
        .ok_or_else(|| anyhow!("Expected buffer"))?
        .as_object()
        .ok_or_else(|| anyhow!("buffer must be an object"))?
        .as_array_buffer()
        .ok_or_else(|| anyhow!("buffer must be an ArrayBuffer"))?
        .as_bytes()
        .ok_or_else(|| anyhow!("Could not interpret buffer as bytes"))?;

    let nrecv_ptr = args
        .get(2)
        .ok_or_else(|| anyhow!("Expected nrecv_ptr"))?
        .as_number()
        .ok_or(Error::msg("nrecv_ptr must be a number"))? as usize;

    let roflags_ptr = args
        .get(3)
        .ok_or_else(|| anyhow!("Expected roflags_ptr"))?
        .as_number()
        .ok_or(Error::msg("roflags_ptr must be a number"))? as usize;

    // Encapsulate buffer into an [IoVec; 1].
    let iovec = [IoVec {
        buf: buffer.as_ptr() as i32,
        buf_len: buffer.len() as u32,
    }];

    let res = unsafe {
        sock_recv(
            sockfd,
            iovec.as_ptr() as i32,
            iovec.len() as i32,
            0,
            nrecv_ptr as i32,
            roflags_ptr as i32,
        )
    };

    if res != 0 {
        bail!("sock_recv failed with code {}", res);
    }

    Ok(Value::new_number(cx, nrecv_ptr as f64))
}

/// Write data to the TCP stream
fn write(args: Args<'_>) -> Result<Value<'_>> {
    let (cx, args) = args.release();
    let sockfd = args
        .get(0)
        .ok_or_else(|| anyhow!("Expected sockfd"))?
        .as_int()
        .ok_or(Error::msg("sockfd must be an integer"))? as i32;

    let buffer = args
        .get(1)
        .ok_or_else(|| anyhow!("Expected buffer"))?
        .as_object()
        .ok_or(Error::msg("buffer must be an object"))?
        .as_array_buffer()
        .ok_or(Error::msg("buffer must be an ArrayBuffer"))?
        .as_bytes()
        .ok_or(Error::msg("Could not interpret buffer as bytes"))?;

    let siflags = args
        .get(2)
        .ok_or_else(|| anyhow!("Expected siflags"))?
        .as_number()
        .ok_or(Error::msg("siflags must be a number"))? as usize;

    let nsent_ptr = args
        .get(3)
        .ok_or_else(|| anyhow!("Expected nsent_ptr"))?
        .as_number()
        .ok_or(Error::msg("nsent_ptr must be a number"))? as usize;

    // Encapsulate buffer into an [IoVec; 1].
    let iovec = [IoVec {
        buf: buffer.as_ptr() as i32,
        buf_len: buffer.len() as u32,
    }];

    let res = unsafe {
        sock_send(
            sockfd,
            iovec.as_ptr() as i32,
            iovec.len() as i32,
            siflags as i32,
            nsent_ptr as i32,
        )
    };

    if res != 0 {
        bail!("sock_send failed with code {}", res);
    }

    Ok(Value::new_number(cx, nsent_ptr as f64))
}
