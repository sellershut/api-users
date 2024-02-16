use anyhow::Result;
use api_database::Client;

use api_core::{api::MutateCategories, User};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use fake::{faker::lorem::en::Words, Fake};
use uuid::Uuid;

async fn create_client(with_ns: Option<&str>) -> Result<Client> {
    let db_host = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let db_host = db_host.replace("http://", "");

    let username = std::env::var("TEST_DATABASE_USERNAME").expect("TEST_DATABASE_USERNAME");
    let password = std::env::var("TEST_DATABASE_PASSWORD").expect("TEST_DATABASE_PASSWORD");
    let db_namespace = std::env::var("TEST_DATABASE_NAMESPACE").expect("TEST_DATABASE_NAMESPACE");
    let db_name = std::env::var("TEST_DATABASE_NAME").expect("TEST_DATABASE_NAME");

    let client = Client::try_new(
        &db_host,
        &username,
        &password,
        with_ns.unwrap_or(&db_namespace),
        &db_name,
        None,
        None,
    )
    .await?;

    Ok(client)
}

fn bench(c: &mut Criterion) {
    dotenvy::dotenv().ok();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = rt.block_on(create_client(Some("benchmarks"))).unwrap();

    let size = 100;

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

    c.bench_with_input(
        BenchmarkId::new("category insert", size),
        &size,
        |b, &_s| {
            b.to_async(&rt)
                .iter(|| black_box(client.create_category(&category)));
        },
    );

    // should probably clean everything after inserting
}

criterion_group!(benches, bench);
criterion_main!(benches);
