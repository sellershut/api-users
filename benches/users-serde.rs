use api_core::{User, UserType};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fake::{
    faker::{
        internet::raw::{FreeEmail, Username},
        phone_number::raw::PhoneNumber,
    },
    locales::EN,
    Fake,
};

use time::OffsetDateTime;
use uuid::Uuid;

fn bench(c: &mut Criterion) {
    let count = 24;

    let users: Vec<_> = (0..count)
        .map(|_| User {
            id: Uuid::now_v7(),
            name: None,
            username: Username(EN).fake(),
            email: FreeEmail(EN).fake(),
            avatar: None,
            user_type: UserType::Individual,
            phone_number: Some(PhoneNumber(EN).fake()),
            created: OffsetDateTime::now_utc(),
            updated: OffsetDateTime::now_utc(),
        })
        .collect();

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
