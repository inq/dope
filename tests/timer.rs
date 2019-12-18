use dope::executor::{self, Executor};
use dope::timer::Timer;

use chrono::Duration;

async fn test_timer_inner(executor: executor::Handle) -> Result<Vec<i32>, failure::Error> {
    use futures::StreamExt;

    let reactor = executor.reactor()?;
    let stream1 = Timer::start(reactor.clone(), Duration::milliseconds(500))?.map(|()| 1i32);
    let stream2 = Timer::start(reactor.clone(), Duration::milliseconds(1375))?.map(|()| 2i32);

    Ok(futures::stream::select(stream1, stream2)
        .take(10)
        .collect()
        .await)
}

#[test]
fn test_timer() -> Result<(), failure::Error> {
    let executor = Executor::new()?;
    let handle = executor.handle();
    let res = executor.block_on(test_timer_inner(handle)).unwrap()?;
    assert_eq!(res, vec![1, 1, 2, 1, 1, 1, 2, 1, 1, 1]);
    Ok(())
}
