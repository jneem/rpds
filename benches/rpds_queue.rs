/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */
#![feature(uniform_paths)]
#![cfg_attr(feature = "fatal-warnings", deny(warnings))]

extern crate criterion;
extern crate rpds;

mod utils;

use ::criterion::{black_box, Criterion, criterion_main, criterion_group};
use ::rpds::Queue;
use crate::utils::limit;

fn rpds_queue_enqueue(c: &mut Criterion) {
    let limit = limit(10_000);

    c.bench_function("rpds queue enqueue", move |b| {
        b.iter(|| {
            let mut queue: Queue<usize> = Queue::new();

            for i in 0..limit {
                queue = queue.enqueue(i);
            }

            queue
        })
    });
}

fn rpds_queue_enqueue_mut(c: &mut Criterion) {
    let limit = limit(10_000);

    c.bench_function("rpds queue enqueue mut", move |b| {
        b.iter(|| {
            let mut queue: Queue<usize> = Queue::new();

            for i in 0..limit {
                queue.enqueue_mut(i);
            }

            queue
        })
    });
}

fn rpds_queue_dequeue(c: &mut Criterion) {
    let limit = limit(10_000);

    c.bench_function("rpds queue dequeue", move |b| {
        b.iter_with_setup(
            || {
                let mut queue: Queue<usize> = Queue::new();

                for i in 0..limit {
                    queue.enqueue_mut(i);
                }

                queue
            },
            |mut queue| {
                for _ in 0..limit {
                    queue = queue.dequeue().unwrap();
                }

                queue
            },
        );
    });
}

fn rpds_queue_dequeue_mut(c: &mut Criterion) {
    let limit = limit(10_000);

    c.bench_function("rpds queue dequeue mut", move |b| {
        b.iter_with_setup(
            || {
                let mut queue: Queue<usize> = Queue::new();

                for i in 0..limit {
                    queue.enqueue_mut(i);
                }

                queue
            },
            |mut queue| {
                for _ in 0..limit {
                    queue.dequeue_mut();
                }

                queue
            },
        );
    });
}

fn rpds_queue_iterate(c: &mut Criterion) {
    let limit = limit(10_000);
    let mut queue: Queue<usize> = Queue::new();

    for i in 0..limit {
        queue.enqueue_mut(i);
    }

    c.bench_function("rpds queue iterate", move |b| {
        b.iter(|| {
            for i in queue.iter() {
                black_box(i);
            }
        })
    });
}

criterion_group!(
    benches,
    rpds_queue_enqueue,
    rpds_queue_enqueue_mut,
    rpds_queue_dequeue,
    rpds_queue_dequeue_mut,
    rpds_queue_iterate
);
criterion_main!(benches);
