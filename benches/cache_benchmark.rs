use criterion::measurement::WallTime;
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use sloth::cache::Cache;
use std::hint::black_box;
use std::iter;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Instant;

// RwLock-based cache for comparison
struct LockCache<T: Clone> {
    data: RwLock<T>,
}

impl<T: Clone> LockCache<T> {
    fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data),
        }
    }

    fn get_data(&self) -> T {
        self.data.read().unwrap().clone()
    }

    fn update(&self, data: T) {
        *self.data.write().unwrap() = data;
    }
}

const JSON: &str = r#""{"timestamp":"2025-12-25T18:12:10Z","version":"2.4.1-alpha","system_config":{"cache_policy":"LRU","replication_factor":3,"clusters":["us-east-1","eu-central-1","ap-southeast-1"],"features":{"compression":true,"encryption":false,"logging":"verbose"}},"data":{"users":[{"id":10293,"uuid":"550e8400-e29b-41d4-a716-446655440000","profile":{"name":"Alex Rivera","email":"arivera@example.com","bio":"Passionate developer and performance engineer focusing on low-latency systems and distributed architecture.","preferences":{"theme":"dark","notifications":{"email":true,"sms":false,"push":true},"language":"en-US"}},"activity_logs":[{"action":"login","ip":"192.168.1.45","device":"Desktop-macOS"},{"action":"update_profile","ip":"192.168.1.45","device":"Desktop-macOS"}],"metadata":{"last_seen":"2025-12-24T22:15:00Z","account_status":"premium","tags":["beta-tester","dev-ops","priority-support"]}},{"id":10294,"uuid":"670f9511-f30c-52e5-b827-557766551111","profile":{"name":"Jordan Smith","email":"jsmith@example.com","bio":"Digital nomad traveling the world while building scalable microservices.","preferences":{"theme":"light","notifications":{"email":false,"sms":true,"push":true},"language":"de-DE"}},"activity_logs":[{"action":"purchase","item_id":9982,"price":299.99}],"metadata":{"last_seen":"2025-12-25T10:05:30Z","account_status":"standard","tags":["traveler","early-adopter"]}}],"inventory":{"categories":["electronics","home-office","books"],"items":[{"sku":"HW-9920-X","name":"UltraWide Monitor 34-inch","specs":{"resolution":"3440x1440","refresh_rate":"144Hz","panel":"IPS"},"stock":{"warehouse_a":45,"warehouse_b":12,"warehouse_c":0},"price_history":[{"date":"2025-01-01","price":599.99},{"date":"2025-06-01","price":549.99}]},{"sku":"BK-1102-Z","name":"Distributed Systems Patterns","author":"Brendan Burns","tags":["tech","education","architecture"],"rating":4.9}]}},"checksum":"a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"}""#;

fn bench_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("reads");

    let reads_per_worker = 100_000;
    for workers in [1, 2, 4] {
        group.throughput(Throughput::Elements(workers * reads_per_worker));

        macro_rules! benchmark {
            ($cache: expr, $name: literal) => {
                group.bench_function(BenchmarkId::new($name, format!("{workers}t")), |b| {
                    b.iter_custom(|iters| {
                        (0..iters)
                            .map(|_| {
                                let cache = $cache;

                                let start: AtomicBool = AtomicBool::new(false);
                                let done_counter: AtomicU8 = AtomicU8::new(0);

                                thread::scope(|s| {
                                    for _ in 0..workers {
                                        s.spawn(|| {
                                            while !start.load(Ordering::Acquire) {
                                                std::hint::spin_loop();
                                            }

                                            for _ in 0..reads_per_worker {
                                                black_box(cache.get_data());
                                            }

                                            done_counter.fetch_add(1, Ordering::Release);
                                        });
                                    }
                                    let time = Instant::now();

                                    start.store(true, Ordering::Release);

                                    while done_counter.load(Ordering::Acquire) != workers as u8 {
                                        std::hint::spin_loop();
                                    }

                                    time.elapsed()
                                })
                            })
                            .sum()
                    });
                });
            };
        }

        benchmark!(Cache::<String, 4>::new(String::from(JSON)), "cache");
        benchmark!(Cache::<String, 8>::new(String::from(JSON)), "cache_8");
        benchmark!(LockCache::<String>::new(String::from(JSON)), "lock");
    }

    group.finish();
}

fn bench_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("writes");

    let writes_per_worker = 50_000;
    for workers in [1, 2, 4] {
        group.throughput(Throughput::Elements(workers * writes_per_worker));

        macro_rules! benchmark {
            ($cache: expr, $name: literal) => {
                group.bench_function(BenchmarkId::new($name, format!("{workers}t")), |b| {
                    b.iter_custom(|iters| {
                        (0..iters)
                            .map(|_| {
                                let cache = $cache;

                                let start: AtomicBool = AtomicBool::new(false);
                                let done_counter: AtomicU8 = AtomicU8::new(0);

                                thread::scope(|s| {
                                    for _ in 0..workers {
                                        s.spawn(|| {
                                            while !start.load(Ordering::Acquire) {
                                                std::hint::spin_loop();
                                            }

                                            for _ in 0..writes_per_worker {
                                                black_box(cache.update(String::from(JSON)));
                                            }

                                            done_counter.fetch_add(1, Ordering::Release);
                                        });
                                    }
                                    let time = Instant::now();

                                    start.store(true, Ordering::Release);

                                    while done_counter.load(Ordering::Acquire) != workers as u8 {
                                        std::hint::spin_loop();
                                    }

                                    time.elapsed()
                                })
                            })
                            .sum()
                    });
                });
            };
        }

        benchmark!(Cache::<String, 4>::new(String::from(JSON)), "cache_4");
        benchmark!(Cache::<String, 8>::new(String::from(JSON)), "cache_8");
        benchmark!(LockCache::<String>::new(String::from(JSON)), "lock");
    }

    group.finish();
}

fn bench_read_and_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_and_writes");

    let reads_per_worker = 100_000;
    let writes_per_worker = 10_000;

    for (writer_workers, readers) in [(1, &[1_u64] as &[u64]), (2, &[2, 4]), (4, &[6, 8])] {
        for &reader_workers in readers {
            group.throughput(Throughput::Elements(
                (reader_workers * reads_per_worker) + (writer_workers * writes_per_worker),
            ));

            macro_rules! benchmark {
                ($cache: expr, $name: literal) => {
                    group.bench_function(
                        BenchmarkId::new($name, format!("{reader_workers}r_{writer_workers}w")),
                        |b| {
                            b.iter_custom(|iters| {
                                (0..iters)
                                    .map(|_| {
                                        let cache = $cache;

                                        let start: AtomicBool = AtomicBool::new(false);
                                        let done_counter: AtomicU8 = AtomicU8::new(0);

                                        thread::scope(|s| {
                                            for _ in 0..reader_workers {
                                                s.spawn(|| {
                                                    while !start.load(Ordering::Acquire) {
                                                        std::hint::spin_loop();
                                                    }

                                                    for _ in 0..reads_per_worker {
                                                        black_box(cache.get_data());
                                                    }

                                                    done_counter.fetch_add(1, Ordering::Release);
                                                });
                                            }

                                            for _ in 0..writer_workers {
                                                s.spawn(|| {
                                                    while !start.load(Ordering::Acquire) {
                                                        std::hint::spin_loop();
                                                    }

                                                    for _ in 0..writes_per_worker {
                                                        black_box(cache.update(String::from(JSON)));
                                                    }

                                                    done_counter.fetch_add(1, Ordering::Release);
                                                });
                                            }
                                            let time = Instant::now();

                                            start.store(true, Ordering::Release);

                                            while done_counter.load(Ordering::Acquire)
                                                != (reader_workers + writer_workers) as u8
                                            {
                                                std::hint::spin_loop();
                                            }

                                            time.elapsed()
                                        })
                                    })
                                    .sum()
                            });
                        },
                    );
                };
            }

            benchmark!(Cache::<String, 4>::new(String::from(JSON)), "cache");
            benchmark!(Cache::<String, 8>::new(String::from(JSON)), "cache_8");
            benchmark!(LockCache::<String>::new(String::from(JSON)), "lock");
        }
    }

    group.finish();
}

criterion_group!(benches, bench_reads, bench_writes, bench_read_and_writes);

criterion_main!(benches);
