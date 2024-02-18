use api_core::{User, UserType};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fake::{faker::lorem::en::Words, Fake};
use time::OffsetDateTime;
use uuid::Uuid;

fn bench(c: &mut Criterion) {
    let count = 24;
    let mut users = Vec::with_capacity(count);

    for _ in 0..count {
        let words: Vec<String> = Words(1..5).fake();
        let words = words.join(" ");

        let user = User {
            id: Uuid::now_v7(),
            name: None,
            username: words,
            email: None,
            avatar: None,
            user_type: UserType::Individual,
            phone_number: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: None,
        };

        users.push(user);
    }

    c.bench_function(&format!("serialise {count}"), |b| {
        b.iter(|| black_box(serde_json::to_string(&users)))
    });

    let cat_str = serde_json::to_string(&users).unwrap();

    c.bench_function(&format!("deserialise {count}"), |b| {
        b.iter(|| black_box(serde_json::from_str::<Vec<User>>(&cat_str)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
