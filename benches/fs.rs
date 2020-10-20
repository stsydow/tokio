#![cfg(unix)]

use tokio::prelude::*;

use tokio::codec::{BytesCodec, FramedRead /*FramedWrite*/};
use tokio::fs::File;

use bencher::{benchmark_group, benchmark_main, Bencher};

use std::fs::File as StdFile;
use std::io::Read as StdRead;

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new()
        .core_threads(2)
        .build()
        .unwrap()
}

const BLOCK_COUNT: usize = 1_000;

const BUFFER_SIZE: usize = 4096;
const DEV_ZERO: &'static str = "/dev/zero";

fn async_read_codec(b: &mut Bencher) {
    let mut runtime = rt();

    b.iter(|| {
        let task = File::open(DEV_ZERO).and_then(|file| {
            let input_stream = FramedRead::new(file, BytesCodec::new());
            input_stream.take(BLOCK_COUNT as u64).for_each(|_| Ok(()))
        });
        runtime.block_on(task).expect("task error");
    });
}

fn async_read(b: &mut Bencher) {
    let mut runtime = rt();

    b.iter(|| {
        let task = File::open(DEV_ZERO).and_then(move |mut file| {
            let mut buffer = [0u8; BUFFER_SIZE];
            stream::poll_fn(move || {
                let r = match file.poll_read(&mut buffer)? {
                    Async::Ready(count) => Async::Ready(Some(count)),
                    Async::NotReady => Async::NotReady,
                };
                Ok(r)
            })
            .take(BLOCK_COUNT as u64)
            .for_each(|_| Ok(()))
        });

        runtime.block_on(task).expect("task error");
    });
}

fn sync_read(b: &mut Bencher) {
    b.iter(|| {
        let mut file = StdFile::open(DEV_ZERO).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];

        for _i in 0..BLOCK_COUNT {
            file.read_exact(&mut buffer).unwrap();
        }
    });
}

benchmark_group!(file, async_read, async_read_codec, sync_read);

benchmark_main!(file);
