use criterion::{
    black_box, criterion_group, criterion_main,
    measurement::{Measurement, WallTime},
    Criterion,
};
#[cfg(target_os = "linux")]
use criterion_linux_perf::{PerfMeasurement, PerfMode};
use memmem::{Searcher, TwoWaySearcher};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn search_short_haystack<M: Measurement>(c: &mut Criterion<M>) {
    let mut needles = BufReader::new(File::open("../data/words.txt").unwrap())
        .lines()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    needles.sort_unstable_by_key(|needle| needle.len());
    let needles = needles.iter().map(String::as_str).collect::<Vec<_>>();

    let mut group = c.benchmark_group("short_haystack");

    group.bench_function("String::find", |b| {
        b.iter(|| {
            for (i, needle) in needles.iter().enumerate() {
                for haystack in &needles[i..] {
                    black_box(haystack.find(needle));
                }
            }
        });
    });

    group.bench_function("memmem::TwoWaySearcher::search_in", |b| {
        let searchers = needles
            .iter()
            .map(|needle| TwoWaySearcher::new(needle.as_bytes()))
            .collect::<Vec<_>>();

        b.iter(|| {
            for (i, searcher) in searchers.iter().enumerate() {
                for haystack in &needles[i..] {
                    black_box(searcher.search_in(haystack.as_bytes()));
                }
            }
        });
    });

    group.bench_function("twoway::find_bytes", |b| {
        b.iter(|| {
            for (i, needle) in needles.iter().enumerate() {
                for haystack in &needles[i..] {
                    black_box(twoway::find_bytes(haystack.as_bytes(), needle.as_bytes()));
                }
            }
        });
    });

    group.bench_function("memchr::memmem::find", |b| {
        b.iter(|| {
            for (i, needle) in needles.iter().enumerate() {
                for haystack in &needles[i..] {
                    black_box(memchr::memmem::find(haystack.as_bytes(), needle.as_bytes()));
                }
            }
        });
    });

    group.bench_function("memchr::memmem::Finder::find", |b| {
        let finders = needles
            .iter()
            .map(|&needle| memchr::memmem::Finder::new(needle.as_bytes()))
            .collect::<Vec<_>>();

        b.iter(|| {
            for (i, finder) in finders.iter().enumerate() {
                for haystack in &needles[i..] {
                    black_box(finder.find(haystack.as_bytes()));
                }
            }
        });
    });

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use sliceslice::x86::{Avx2Searcher, DynamicAvx2Searcher};

        #[cfg(feature = "sse4-strstr")]
        group.bench_function("sse4_strstr::avx2_strstr_v2", |b| {
            b.iter(|| {
                for (i, needle) in needles.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(unsafe {
                            sse4_strstr::avx2_strstr_v2(haystack.as_bytes(), needle.as_bytes())
                        });
                    }
                }
            });
        });

        group.bench_function("Avx2Searcher::search_in", |b| {
            let searchers = needles
                .iter()
                .map(|&needle| unsafe { Avx2Searcher::new(needle.as_bytes()) })
                .collect::<Vec<_>>();

            b.iter(|| {
                for (i, searcher) in searchers.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(unsafe { searcher.search_in(haystack.as_bytes()) });
                    }
                }
            });
        });

        group.bench_function("DynamicAvx2Searcher::search_in", |b| {
            let searchers = needles
                .iter()
                .map(|&needle| unsafe { DynamicAvx2Searcher::new(needle.as_bytes()) })
                .collect::<Vec<_>>();

            b.iter(|| {
                for (i, searcher) in searchers.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(unsafe { searcher.search_in(haystack.as_bytes()) });
                    }
                }
            });
        });
    }

    #[cfg(feature = "stdsimd")]
    {
        use sliceslice::stdsimd::StdSimdSearcher;

        group.bench_function("StdSimdSearcher::search_in", |b| {
            let searchers = needles
                .iter()
                .map(|&needle| StdSimdSearcher::new(needle.as_bytes()))
                .collect::<Vec<_>>();

            b.iter(|| {
                for (i, searcher) in searchers.iter().enumerate() {
                    for haystack in &needles[i..] {
                        black_box(searcher.search_in(haystack.as_bytes()));
                    }
                }
            });
        });
    }

    group.finish();
}

fn search_haystack<M: Measurement>(
    c: &mut Criterion<M>,
    name: &'static str,
    haystack: &'static [u8],
) {
    let needles = BufReader::new(File::open("../data/words.txt").unwrap())
        .lines()
        .map(Result::unwrap)
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group(name);

    group.bench_function("String::find", |b| {
        let haystack = String::from_utf8_lossy(haystack).into_owned();
        b.iter(|| {
            for needle in &needles {
                black_box(haystack.find(needle));
            }
        });
    });

    group.bench_function("memmem::TwoWaySearcher::search_in", |b| {
        let searchers = needles
            .iter()
            .map(|needle| TwoWaySearcher::new(needle.as_bytes()))
            .collect::<Vec<_>>();

        b.iter(|| {
            for searcher in &searchers {
                black_box(searcher.search_in(haystack));
            }
        });
    });

    group.bench_function("twoway::find_bytes", |b| {
        b.iter(|| {
            for needle in &needles {
                black_box(twoway::find_bytes(haystack, needle.as_bytes()));
            }
        });
    });

    group.bench_function("memchr::memmem::find", |b| {
        b.iter(|| {
            for needle in &needles {
                black_box(memchr::memmem::find(haystack, needle.as_bytes()));
            }
        });
    });

    group.bench_function("memchr::memmem::Finder::find", |b| {
        let finders = needles
            .iter()
            .map(|needle| memchr::memmem::Finder::new(needle.as_bytes()))
            .collect::<Vec<_>>();

        b.iter(|| {
            for finder in &finders {
                black_box(finder.find(haystack));
            }
        });
    });

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use sliceslice::x86::{Avx2Searcher, DynamicAvx2Searcher};

        #[cfg(feature = "sse4-strstr")]
        group.bench_function("sse4_strstr::avx2_strstr_v2", |b| {
            b.iter(|| {
                for needle in &needles {
                    black_box(unsafe { sse4_strstr::avx2_strstr_v2(haystack, needle.as_bytes()) });
                }
            });
        });

        group.bench_function("Avx2Searcher::search_in", |b| {
            let searchers = needles
                .iter()
                .map(|needle| unsafe { Avx2Searcher::new(needle.as_bytes()) })
                .collect::<Vec<_>>();

            b.iter(|| {
                for searcher in &searchers {
                    black_box(unsafe { searcher.search_in(haystack) });
                }
            });
        });

        group.bench_function("DynamicAvx2Searcher::search_in", |b| {
            let searchers = needles
                .iter()
                .map(|needle| unsafe { DynamicAvx2Searcher::new(needle.as_bytes()) })
                .collect::<Vec<_>>();

            b.iter(|| {
                for searcher in &searchers {
                    black_box(unsafe { searcher.search_in(haystack) });
                }
            });
        });
    }

    #[cfg(feature = "stdsimd")]
    {
        use sliceslice::stdsimd::StdSimdSearcher;

        group.bench_function("StdSimdSearcher::search_in", |b| {
            let searchers = needles
                .iter()
                .map(|needle| StdSimdSearcher::new(needle.as_bytes()))
                .collect::<Vec<_>>();

            b.iter(|| {
                for searcher in &searchers {
                    black_box(searcher.search_in(haystack));
                }
            });
        });
    }

    group.finish();
}

fn search_long_haystack<M: Measurement>(c: &mut Criterion<M>) {
    let haystack = include_bytes!("../../data/i386.txt");
    search_haystack(c, "long_haystack", haystack)
}

fn search_random_haystack<M: Measurement>(c: &mut Criterion<M>) {
    let haystack = include_bytes!("../../data/haystack");
    search_haystack(c, "random_haystack", haystack)
}

criterion_group!(
    name = i386_wall_time;
    config = Criterion::default().with_measurement(WallTime);
    targets = search_short_haystack, search_long_haystack, search_random_haystack
);

#[cfg(target_os = "linux")]
criterion_group!(
    name = i386_perf_instructions;
    config = Criterion::default().with_measurement(PerfMeasurement::new(PerfMode::Instructions));
    targets = search_short_haystack, search_long_haystack, search_random_haystack
);

#[cfg(target_os = "linux")]
criterion_main!(i386_wall_time, i386_perf_instructions);

#[cfg(not(target_os = "linux"))]
criterion_main!(i386_wall_time);
