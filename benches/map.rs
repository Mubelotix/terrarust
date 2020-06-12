use criterion::{criterion_group, criterion_main, Criterion};


#[cfg(target_arch = "wasm32")]
pub fn map_initialization(_c: &mut Criterion) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn map_initialization(c: &mut Criterion) {
    use terrarust::{map::*, blocks::*, chunks::*};
    use criterion::black_box;

    fn function(_useless: usize) {
        let mut map = Map {
            chunks: Vec::new(),
            first_chunk_number: -5,
            first_block: 0,
            air: Block {
                block_type: BlockType::Air,
                natural_background: NaturalBackground::Sky,
                light: 0,
            },
            to_update_chunks: Vec::new(),
            light_update: Vec::new(),
        };

        let mut height: f64 = 20.0;
        let mut slope: f64 = 0.2;
        for i in -5..5 {

            map.chunks.push((
                Chunk::generate(&mut height, &mut slope, true, i * 32),
                (),
                (),
            ));
        }

        map.init_lights();
    }
    c.bench_function("map initialization", |b| b.iter(|| function(black_box(0))));
}

criterion_group!(benches, map_initialization);
criterion_main!(benches);