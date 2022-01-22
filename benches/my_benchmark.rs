//    Google Keep to Turtl
//    Copyright (C) 2022 Panikon and Magnus Manske
//
//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see <http://www.gnu.org/licenses/>.

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