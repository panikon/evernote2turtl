use criterion::{black_box, criterion_group, criterion_main, Criterion};
//use keep2turtl::create_turtl_backup_from_directory;
extern crate keep2turtl;

pub fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function("testing", |b| {
						b.iter(|| {
							keep2turtl::create_turtl_backup_from_directory("Keepx", 584);
						});
					});
	//let j = keep2turtl::create_turtl_backup_from_directory("Keepx", 584, "turtl", true).unwrap();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);