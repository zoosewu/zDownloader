//! One task's transfer: probe the server, fetch the missing bytes, finalize the file.
//!
//! Ranged mode (server honours `Range` and reports a size): the missing byte ranges of the
//! part file are cut into segments and fetched by up to `connections` workers. Each segment
//! retries from where it stopped, so a dropped connection costs only the in-flight bytes.
//!
//! Stream mode (no range support or unknown size): one GET from byte 0; a failure restarts it.
//!
//! Cancelling keeps a ranged part file for a later resume; the caller decides whether to
//! discard it.

use crate::connection::{ConnectionSource, Lease, Outcome};
use crate::error::CoreError;
use crate::part_file::{PartFile, Validator};
use crate::rate::RateLimiter;
use crate::task::ActiveRange;
use futures_util::StreamExt;
use reqwest::header::{CONTENT_RANGE, ETAG, LAST_MODIFIED, RANGE};
use reqwest::{Response, StatusCode};
use std::collections::{BTreeMap, VecDeque};
use std::future::Future;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct TransferSpec {
    /// The task's own URL. Identifies the resource for resuming, so a part file survives
    /// the direct link changing between resolutions.
    pub origin: String,
    /// What to fetch: the resolved direct link.
    pub url: String,
    /// Sent with every request (from resolution: Referer, Cookie, ...).
    pub headers: Vec<(String, String)>,
    /// Directory and file name of the finished file (renamed on conflict).
    pub dir: PathBuf,
    pub name: String,
    /// In-progress file; see `naming::part_path`.
    pub part_path: PathBuf,
    pub connections: usize,
    pub segment_size: u64,
    pub retry: RetryPolicy,
}

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Attempts per segment (and per stream / probe) without progress before giving up.
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 5, base_delay: Duration::from_secs(1), max_delay: Duration::from_secs(30) }
    }
}

impl RetryPolicy {
    /// Delay before retry number `attempt` (1-based): base, 2×base, 4×base, … capped.
    pub fn delay(&self, attempt: u32) -> Duration {
        let factor = 1u32.checked_shl(attempt.saturating_sub(1)).unwrap_or(u32::MAX);
        self.base_delay.saturating_mul(factor).min(self.max_delay)
    }
}

/// Live counters the manager samples for progress and speed, plus the segment picture.
pub struct Progress {
    downloaded: AtomicU64,
    total: AtomicU64,
    /// The part file while a ranged transfer runs; `None` in stream mode or between attempts.
    part: Mutex<Option<Arc<PartFile>>>,
    /// What each worker is fetching right now, by worker index.
    active: Mutex<BTreeMap<usize, ActiveRange>>,
}

const UNKNOWN: u64 = u64::MAX;

impl Default for Progress {
    fn default() -> Self {
        Self {
            downloaded: AtomicU64::new(0),
            total: AtomicU64::new(UNKNOWN),
            part: Mutex::new(None),
            active: Mutex::new(BTreeMap::new()),
        }
    }
}

impl Progress {
    /// Finished and in-flight ranges of a running ranged transfer; `None` in stream mode.
    pub fn segments(&self) -> Option<(Vec<(u64, u64)>, Vec<ActiveRange>)> {
        let done = self.part.lock().unwrap().as_ref()?.done_ranges();
        let active = self.active.lock().unwrap().values().copied().collect();
        Some((done, active))
    }

    fn set_part(&self, part: Option<Arc<PartFile>>) {
        *self.part.lock().unwrap() = part;
        self.active.lock().unwrap().clear();
    }

    fn set_active(&self, worker: usize, range: Option<ActiveRange>) {
        let mut active = self.active.lock().unwrap();
        match range {
            Some(r) => active.insert(worker, r),
            None => active.remove(&worker),
        };
    }

    pub fn downloaded(&self) -> u64 {
        self.downloaded.load(Ordering::Relaxed)
    }

    pub fn total(&self) -> Option<u64> {
        Some(self.total.load(Ordering::Relaxed)).filter(|&t| t != UNKNOWN)
    }

    fn reset(&self, downloaded: u64, total: Option<u64>) {
        self.downloaded.store(downloaded, Ordering::Relaxed);
        self.total.store(total.unwrap_or(UNKNOWN), Ordering::Relaxed);
    }

    fn add(&self, n: u64) {
        self.downloaded.fetch_add(n, Ordering::Relaxed);
    }
}

/// Everything a transfer shares with the rest of the system.
#[derive(Clone)]
pub struct Ctx {
    pub conn: Arc<dyn ConnectionSource>,
    pub rate: Arc<RateLimiter>,
    pub progress: Arc<Progress>,
    pub cancel: CancellationToken,
}

/// Run the transfer to completion. Returns the final file path.
pub async fn run(spec: &TransferSpec, ctx: &Ctx) -> Result<PathBuf, CoreError> {
    let probe = with_retry(spec.retry, &ctx.cancel, || probe(spec, ctx)).await?;
    match probe {
        Probe::Ranged(validator) => match ranged(spec, ctx, validator).await {
            Err(SegmentError::RangeIgnored) => stream(spec, ctx, None).await,
            Err(SegmentError::Core(e)) => Err(e),
            Ok(path) => Ok(path),
        },
        Probe::Whole(response) => stream(spec, ctx, Some(response)).await,
    }
}

enum Probe {
    Ranged(Validator),
    /// The server ignored the range; this response already carries the whole body.
    Whole(Response),
}

async fn probe(spec: &TransferSpec, ctx: &Ctx) -> Result<Probe, CoreError> {
    let (lease, response) = send(ctx, spec, Some("bytes=0-0".into())).await?;
    let header = |name| response.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_string);
    let validator =
        |size| Validator { url: spec.origin.clone(), size, etag: header(ETAG), last_modified: header(LAST_MODIFIED) };

    match response.status() {
        StatusCode::PARTIAL_CONTENT => match header(CONTENT_RANGE).as_deref().and_then(parse_content_range) {
            Some(ContentRange { total: Some(total), .. }) => Ok(Probe::Ranged(validator(Some(total)))),
            // Ranges work but the size is unknown: cannot plan segments.
            _ => Ok(Probe::Whole(send(ctx, spec, None).await?.1)),
        },
        StatusCode::OK => Ok(Probe::Whole(response)),
        StatusCode::RANGE_NOT_SATISFIABLE if header(CONTENT_RANGE).is_some_and(|v| v.ends_with("/0")) => {
            Ok(Probe::Ranged(validator(Some(0))))
        }
        status => {
            ctx.conn.report(&lease, Outcome::Failure);
            Err(CoreError::Http(status.as_u16()))
        }
    }
}

/// Send a GET for the spec's source through the connection source, scoring the lease by
/// time to first byte.
async fn send(ctx: &Ctx, spec: &TransferSpec, range: Option<String>) -> Result<(Lease, Response), CoreError> {
    let lease = ctx.conn.acquire()?;
    let mut request = lease.client.get(&spec.url);
    for (name, value) in &spec.headers {
        request = request.header(name.as_str(), value.as_str());
    }
    if let Some(range) = range {
        request = request.header(RANGE, range);
    }
    let started = Instant::now();
    let sent = tokio::select! {
        _ = ctx.cancel.cancelled() => return Err(CoreError::Cancelled),
        r = request.send() => r,
    };
    match sent {
        Ok(response) => {
            ctx.conn.report(&lease, Outcome::Success { latency: started.elapsed() });
            Ok((lease, response))
        }
        Err(e) => {
            ctx.conn.report(&lease, Outcome::Failure);
            Err(e.into())
        }
    }
}

enum SegmentError {
    /// A ranged request came back as 200: the server stopped honouring ranges.
    RangeIgnored,
    Core(CoreError),
}

impl From<CoreError> for SegmentError {
    fn from(e: CoreError) -> Self {
        SegmentError::Core(e)
    }
}

impl From<std::io::Error> for SegmentError {
    fn from(e: std::io::Error) -> Self {
        SegmentError::Core(e.into())
    }
}

async fn ranged(spec: &TransferSpec, ctx: &Ctx, validator: Validator) -> Result<PathBuf, SegmentError> {
    let part = match PartFile::resume(spec.part_path.clone(), &validator)? {
        Some(part) => part,
        None => {
            PartFile::discard(&spec.part_path)?;
            PartFile::create(spec.part_path.clone(), validator)?
        }
    };
    ctx.progress.reset(part.downloaded(), part.size());
    let part = Arc::new(part);
    ctx.progress.set_part(Some(part.clone()));
    let result = run_workers(spec, ctx, &part).await;
    // Release the shared handle before flushing, finalizing or deleting the file.
    ctx.progress.set_part(None);
    // Keep whatever was fetched for the next attempt, whether we failed or not.
    part.flush()?;
    if let Err(e) = result {
        if matches!(e, SegmentError::RangeIgnored) {
            drop(part);
            PartFile::discard(&spec.part_path)?;
        }
        return Err(e);
    }

    let part = Arc::try_unwrap(part).map_err(|_| CoreError::Protocol("part file still shared".into()))?;
    Ok(part.finalize(&spec.dir, &spec.name)?)
}

/// Fetch every missing range of `part` with up to `spec.connections` workers.
async fn run_workers(spec: &TransferSpec, ctx: &Ctx, part: &Arc<PartFile>) -> Result<(), SegmentError> {
    let queue: VecDeque<Range<u64>> = part
        .missing()
        .into_iter()
        .flat_map(|gap| split(gap, spec.segment_size.max(1)))
        .collect();
    let workers = spec.connections.max(1).min(queue.len());
    let queue = Arc::new(Mutex::new(queue));

    let mut set = JoinSet::new();
    let spec = Arc::new(spec.clone());
    for id in 0..workers {
        let (queue, part, ctx, spec) = (queue.clone(), part.clone(), ctx.clone(), spec.clone());
        set.spawn(async move {
            let result = worker(id, &spec, &queue, &part, &ctx).await;
            ctx.progress.set_active(id, None);
            result
        });
    }
    let mut failure = None;
    while let Some(joined) = set.join_next().await {
        let result = joined.unwrap_or_else(|e| Err(CoreError::Protocol(format!("worker crashed: {e}")).into()));
        if let Err(e) = result {
            set.abort_all();
            failure.get_or_insert(e);
        }
    }
    failure.map_or(Ok(()), Err)
}

fn split(range: Range<u64>, size: u64) -> impl Iterator<Item = Range<u64>> {
    (range.start..range.end)
        .step_by(size as usize)
        .map(move |start| start..(start + size).min(range.end))
}

/// Show what `worker` is fetching (or waiting to retry) in the segment picture.
fn mark(ctx: &Ctx, worker: usize, range: &Range<u64>, retrying: bool) {
    ctx.progress.set_active(worker, Some(ActiveRange { start: range.start, end: range.end, retrying }));
}

async fn worker(
    id: usize,
    spec: &TransferSpec,
    queue: &Mutex<VecDeque<Range<u64>>>,
    part: &PartFile,
    ctx: &Ctx,
) -> Result<(), SegmentError> {
    loop {
        let Some(mut range) = queue.lock().unwrap().pop_front() else { return Ok(()) };
        let (retry, mut attempt) = (spec.retry, 0);
        loop {
            mark(ctx, id, &range, false);
            let (result, reached) = fetch_range(spec, range.clone(), part, ctx).await;
            match result {
                Ok(()) => break,
                Err(SegmentError::Core(e)) if e.is_retryable() => {
                    // Progress earns a fresh set of attempts; only a stuck segment gives up.
                    attempt = if reached > range.start { 1 } else { attempt + 1 };
                    if attempt >= retry.max_attempts {
                        return Err(e.into());
                    }
                    range.start = reached;
                    mark(ctx, id, &range, true);
                    sleep_or_cancel(retry.delay(attempt), &ctx.cancel).await?;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Fetch `range` into the part file. Also returns how far it got, for resuming the segment.
async fn fetch_range(spec: &TransferSpec, range: Range<u64>, part: &PartFile, ctx: &Ctx) -> (Result<(), SegmentError>, u64) {
    let mut offset = range.start;
    let result = async {
        let (lease, response) = send(ctx, spec, Some(format!("bytes={}-{}", range.start, range.end - 1))).await?;
        match response.status() {
            StatusCode::PARTIAL_CONTENT => {}
            StatusCode::OK => return Err(SegmentError::RangeIgnored),
            status => {
                ctx.conn.report(&lease, Outcome::Failure);
                return Err(CoreError::Http(status.as_u16()).into());
            }
        }
        let content_range = response.headers().get(CONTENT_RANGE).and_then(|v| v.to_str().ok());
        match content_range.and_then(parse_content_range) {
            Some(cr) if cr.start == range.start && cr.end_inclusive == range.end - 1 => {}
            _ => {
                return Err(CoreError::Protocol(format!(
                    "asked for bytes {}-{}, got Content-Range {:?}",
                    range.start,
                    range.end - 1,
                    content_range
                ))
                .into())
            }
        }

        let mut body = response.bytes_stream();
        while offset < range.end {
            let chunk = tokio::select! {
                _ = ctx.cancel.cancelled() => return Err(CoreError::Cancelled.into()),
                next = body.next() => match next {
                    Some(chunk) => chunk.map_err(CoreError::from)?,
                    None => return Err(CoreError::Network("connection closed before segment end".into()).into()),
                },
            };
            let len = chunk.len().min((range.end - offset) as usize);
            ctx.rate.acquire(len as u64).await;
            part.write_at(offset, &chunk[..len])?;
            offset += len as u64;
            ctx.progress.add(len as u64);
        }
        Ok(())
    }
    .await;
    (result, offset)
}

/// Whole-body download from byte 0. Any failure restarts it, since the server cannot resume.
async fn stream(spec: &TransferSpec, ctx: &Ctx, mut first: Option<Response>) -> Result<PathBuf, CoreError> {
    let mut attempt = 0;
    loop {
        let result = async {
            let response = match first.take() {
                Some(r) => r,
                None => send(ctx, spec, None).await?.1,
            };
            if response.status() != StatusCode::OK {
                return Err(CoreError::Http(response.status().as_u16()));
            }
            let size = response.content_length();
            let validator = Validator { url: spec.origin.clone(), size, etag: None, last_modified: None };
            PartFile::discard(&spec.part_path)?;
            let part = PartFile::create(spec.part_path.clone(), validator)?;
            ctx.progress.reset(0, size);

            let mut offset = 0u64;
            let mut body = response.bytes_stream();
            loop {
                let chunk = tokio::select! {
                    _ = ctx.cancel.cancelled() => return Err(CoreError::Cancelled),
                    next = body.next() => match next {
                        Some(chunk) => chunk?,
                        None => break,
                    },
                };
                ctx.rate.acquire(chunk.len() as u64).await;
                part.write_at(offset, &chunk)?;
                offset += chunk.len() as u64;
                ctx.progress.add(chunk.len() as u64);
            }
            if size.is_some_and(|s| offset != s) {
                return Err(CoreError::Network(format!("connection closed at {offset} of {size:?} bytes")));
            }
            Ok(part.finalize(&spec.dir, &spec.name)?)
        }
        .await;

        match result {
            Err(e) if e.is_retryable() && attempt + 1 < spec.retry.max_attempts => {
                attempt += 1;
                sleep_or_cancel(spec.retry.delay(attempt), &ctx.cancel).await?;
            }
            other => return other,
        }
    }
}

async fn with_retry<T, F, Fut>(retry: RetryPolicy, cancel: &CancellationToken, mut f: F) -> Result<T, CoreError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, CoreError>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Err(e) if e.is_retryable() && attempt + 1 < retry.max_attempts => {
                attempt += 1;
                sleep_or_cancel(retry.delay(attempt), cancel).await?;
            }
            other => return other,
        }
    }
}

async fn sleep_or_cancel(delay: Duration, cancel: &CancellationToken) -> Result<(), CoreError> {
    tokio::select! {
        _ = cancel.cancelled() => Err(CoreError::Cancelled),
        _ = tokio::time::sleep(delay) => Ok(()),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ContentRange {
    start: u64,
    end_inclusive: u64,
    total: Option<u64>,
}

/// Parse `bytes START-END/TOTAL` (TOTAL may be `*`).
fn parse_content_range(value: &str) -> Option<ContentRange> {
    let rest = value.trim().strip_prefix("bytes ")?;
    let (span, total) = rest.split_once('/')?;
    let (start, end) = span.split_once('-')?;
    Some(ContentRange {
        start: start.trim().parse().ok()?,
        end_inclusive: end.trim().parse().ok()?,
        total: match total.trim() {
            "*" => None,
            t => Some(t.parse().ok()?),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_content_range() {
        assert_eq!(
            parse_content_range("bytes 0-0/1234"),
            Some(ContentRange { start: 0, end_inclusive: 0, total: Some(1234) })
        );
        assert_eq!(parse_content_range("bytes 5-9/*").unwrap().total, None);
        assert_eq!(parse_content_range("bytes */0"), None);
        assert_eq!(parse_content_range("items 0-1/2"), None);
    }

    #[test]
    fn splits_into_segments() {
        assert_eq!(split(0..10, 4).collect::<Vec<_>>(), vec![0..4, 4..8, 8..10]);
        assert_eq!(split(3..5, 100).collect::<Vec<_>>(), vec![3..5]);
    }

    #[test]
    fn retry_delay_backs_off_and_caps() {
        let p = RetryPolicy { max_attempts: 9, base_delay: Duration::from_secs(1), max_delay: Duration::from_secs(10) };
        assert_eq!(p.delay(1), Duration::from_secs(1));
        assert_eq!(p.delay(2), Duration::from_secs(2));
        assert_eq!(p.delay(3), Duration::from_secs(4));
        assert_eq!(p.delay(5), Duration::from_secs(10));
        assert_eq!(p.delay(64), Duration::from_secs(10));
    }
}
