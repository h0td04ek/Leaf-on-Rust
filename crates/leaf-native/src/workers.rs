use std::io;
use std::ops::Range;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::thread;

type Job = Box<dyn FnOnce() + Send>;

pub struct Workers {
    queues: Vec<mpsc::SyncSender<Job>>,
}

pub fn available_threads() -> usize {
    thread::available_parallelism().map_or(1, |count| count.get())
}

impl Workers {
    pub fn new(threads: usize) -> io::Result<Self> {
        let queues = (1..threads)
            .map(|index| {
                let (send, receive) = mpsc::sync_channel::<Job>(1);
                thread::Builder::new()
                    .name(format!("leaf-native-worker-{index}"))
                    .spawn(move || {
                        while let Ok(job) = receive.recv() {
                            job();
                        }
                    })?;
                Ok(send)
            })
            .collect::<io::Result<_>>()?;
        Ok(Self { queues })
    }

    pub fn can_split(&self, len: usize, min_batch: usize) -> bool {
        len > min_batch && !self.queues.is_empty()
    }

    pub fn map_ranges<T, F>(&self, len: usize, min_batch: usize, work: F) -> Vec<T>
    where
        T: Send + 'static,
        F: Fn(Range<usize>) -> T + Send + Sync + 'static,
    {
        if len == 0 {
            return Vec::new();
        }
        let lanes = len.div_ceil(min_batch).min(self.queues.len() + 1);
        if lanes == 1 {
            return vec![work(0..len)];
        }

        let next = AtomicUsize::new(0);
        let take_partition = Arc::new(move || {
            let index = next.fetch_add(1, Ordering::Relaxed);
            (index < lanes).then(|| (index, work(index * len / lanes..(index + 1) * len / lanes)))
        });
        let (send, receive) = mpsc::channel();
        for queue in self.queues.iter().take(lanes - 1) {
            let take_partition = Arc::clone(&take_partition);
            let send = send.clone();
            let _ = queue.try_send(Box::new(move || {
                if let Some(result) = take_partition() {
                    let _ = send.send(result);
                }
            }));
        }
        drop(send);
        let mut results = Vec::with_capacity(lanes);
        // Keep working while background threads wake up, and wait only for
        // partitions already claimed by a worker.
        results.extend(std::iter::from_fn(|| take_partition()));
        results.extend(receive.into_iter().take(lanes - results.len()));
        assert_eq!(
            results.len(),
            lanes,
            "a world worker failed to finish its work"
        );
        results.sort_unstable_by_key(|(index, _)| *index);
        results.into_iter().map(|(_, result)| result).collect()
    }
}
