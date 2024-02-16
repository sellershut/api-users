use api_core::User;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fake::{faker::lorem::en::Words, Fake};
use uuid::Uuid;

fn bench(c: &mut Criterion) {
    let count = 24;
    let mut categories = Vec::with_capacity(count);

    for _ in 0..count {
        let words: Vec<String> = Words(1..5).fake();
        let words = words.join(" ");

        let sub_categories: Vec<_> = [0; 4].iter().map(|_| Uuid::now_v7()).collect();

        let category = User {
            id: Uuid::now_v7(),
            name: words,
            sub_categories,
            image_url: None,
            parent_id: None,
        };

        categories.push(category);
    }

    c.bench_function(&format!("serialise {count}"), |b| {
        b.iter(|| black_box(serde_json::to_string(&categories)))
    });

    let cat_str = serde_json::to_string(&categories).unwrap();

    c.bench_function(&format!("deserialise {count}"), |b| {
        b.iter(|| black_box(serde_json::from_str::<Vec<User>>(&cat_str)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
