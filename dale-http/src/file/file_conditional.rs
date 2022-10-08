use bytes::{Bytes, BytesMut};
use futures_core::{ready, Stream};
use futures_io::{AsyncRead, AsyncSeek};
use futures_util::{future, stream, AsyncSeekExt, FutureExt, StreamExt};
use headers::{AcceptRanges, ContentLength, ContentType};
use headers::{ContentRange, Range};
use http::{Response, StatusCode};
use std::io;
use std::pin::Pin;
use std::{fs::Metadata, task::Poll};

use crate::{Body, Error, Modifier, Set, With};

use super::file_options::{Cond, FileOptions};

pub(crate) fn file_conditional<A: AsyncRead + AsyncSeek + Send, B>(
    file: A,
    mime: mime::Mime,
    meta: Metadata,
    conditionals: FileOptions,
) -> Result<Response<B>, Error>
where
    A: 'static + std::marker::Unpin,
    B: Body + Modifier<Response<B>>,
{
    let mut len = meta.len();
    let modified = None; // meta.modified().ok().map(LastModified::from);

    let resp = match conditionals.check(modified) {
        Cond::NoBody(resp) => resp,
        Cond::WithBody(range) => {
            bytes_range(range, len)
                .map(move |(start, end)| {
                    let sub_len = end - start;
                    let buf_size = optimal_buf_size(&meta);
                    let stream = file_stream(file, buf_size, (start, end));
                    let body = Body::from_stream(stream);

                    let mut resp = Response::new(body).set(StatusCode::OK);

                    if sub_len != len {
                        resp = resp
                            .set(StatusCode::PARTIAL_CONTENT)
                            .set(ContentRange::bytes(start..end, len).expect("valid ContentRange"));

                        len = sub_len;
                    }
                    // let mime = mime_guess::from_path(path.as_ref()).first_or_octet_stream();

                    resp = resp
                        .set(ContentLength(len))
                        .set(ContentType::from(mime))
                        .set(AcceptRanges::bytes());

                    if let Some(last_modified) = modified {
                        resp = resp.set(last_modified);
                    }

                    resp
                })
                .unwrap_or_else(|BadRange| {
                    // bad byte range
                    Response::with(B::empty())
                        .set(StatusCode::RANGE_NOT_SATISFIABLE)
                        .set(ContentRange::unsatisfied_bytes(len))
                })
        }
    };

    Ok(resp)
}

struct BadRange;

fn bytes_range(range: Option<Range>, max_len: u64) -> Result<(u64, u64), BadRange> {
    use std::ops::Bound;

    let range = if let Some(range) = range {
        range
    } else {
        return Ok((0, max_len));
    };

    let ret = range
        .iter()
        .map(|(start, end)| {
            let start = match start {
                Bound::Unbounded => 0,
                Bound::Included(s) => s,
                Bound::Excluded(s) => s + 1,
            };

            let end = match end {
                Bound::Unbounded => max_len,
                Bound::Included(s) => s + 1,
                Bound::Excluded(s) => s,
            };

            if start < end && end <= max_len {
                Ok((start, end))
            } else {
                log::trace!("unsatisfiable byte range: {}-{}/{}", start, end, max_len);
                Err(BadRange)
            }
        })
        .next()
        .unwrap_or(Ok((0, max_len)));
    ret
}

fn file_stream<F: AsyncRead + AsyncSeek + std::marker::Unpin + Send>(
    mut file: F,
    buf_size: usize,
    (start, end): (u64, u64),
) -> impl Stream<Item = Result<Bytes, io::Error>> + Send {
    use std::io::SeekFrom;

    let seek = async move {
        if start != 0 {
            file.seek(SeekFrom::Start(start)).await?;
        }
        Ok(file)
    };

    seek.into_stream()
        .map(move |result| {
            let mut buf = BytesMut::new();
            buf.resize(buf_size, 0);
            let mut len = end - start;
            let mut f = match result {
                Ok(f) => f,
                Err(f) => return future::Either::Left(stream::once(future::err(f))),
            };

            future::Either::Right(stream::poll_fn(move |cx| {
                if len == 0 {
                    return Poll::Ready(None);
                }
                reserve_at_least(&mut buf, buf_size);

                let mut read_buf = &mut buf; //ReadBuf::new(&mut buf);
                let size = match ready!(Pin::new(&mut f).poll_read(cx, &mut read_buf)) {
                    Ok(s) => s,
                    Err(err) => {
                        log::debug!("file read error: {}", err);
                        return Poll::Ready(Some(Err(err)));
                    }
                };

                if size == 0 {
                    return Poll::Ready(None);
                }

                // if read_buf.filled().is_empty() {
                //     log::debug!("file read found EOF before expected length");
                //     return Poll::Ready(None);
                // }

                let n = size as u64; //read_buf.filled().len() as u64;

                let mut chunk = buf.split().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(chunk)))
            }))
        })
        .flatten()
}

fn reserve_at_least(buf: &mut BytesMut, cap: usize) {
    // println!("RESERIVE {} {} {}", buf.capacity(), buf.len(), cap);
    // FIXME:
    if buf.capacity() - buf.len() < cap || buf.len() < cap {
        buf.reserve(cap);
        unsafe {
            buf.set_len(cap);
        }
    }
}

const DEFAULT_READ_BUF_SIZE: usize = 8_192;

fn optimal_buf_size(metadata: &Metadata) -> usize {
    std::cmp::min(metadata.len() as usize, DEFAULT_READ_BUF_SIZE)
}
