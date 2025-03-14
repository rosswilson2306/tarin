use axum::response::sse::{Event, KeepAlive, Sse};
use std::{convert::Infallible, time::Duration};
use tokio::time::interval;
use tokio_stream::StreamExt as _;

pub async fn sse_reports_handler() -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(interval(Duration::from_secs(2)))
        .map(|_| Ok(Event::default().data("New report available")));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
